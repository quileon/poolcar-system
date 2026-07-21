use crate::auth::AuthenticatedUser;
use crate::entities::{cars, trackers};
use askama::Template;
use askama_web::WebTemplate;
use rocket::form::Form;
use rocket::http::Status;
use rocket::{FromForm, State};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, PaginatorTrait, Set};

pub struct CarWithTracker {
    pub car: cars::Model,
    pub tracker: Option<trackers::Model>,
}

#[derive(Template, WebTemplate)]
#[template(path = "cars.j2")]
pub struct CarsTemplate {
    pub username: String,
    pub role: String,
    pub cars: Vec<CarWithTracker>,
    pub available_trackers: Vec<trackers::Model>,
    pub editing_car: Option<cars::Model>,
    pub current_page: u64,
    pub total_pages: u64,
    pub pages: Vec<u64>,
    pub error: Option<String>,
}

#[derive(FromForm)]
pub struct CarForm<'r> {
    pub name: &'r str,
    pub police_number: &'r str,
    pub car_type: &'r str,
    pub tracker_id: Option<i32>,
}

async fn render_cars(
    db: &DatabaseConnection,
    user: &AuthenticatedUser,
    edit: Option<i32>,
    page: Option<u64>,
    error: Option<String>,
) -> Result<CarsTemplate, Status> {
    let current_page = page.unwrap_or(1);
    let page_size = 5;

    // Fetch the editing car if requested
    let editing_car = match edit {
        Some(id) => cars::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|_| Status::InternalServerError)?,
        None => None,
    };

    // Find busy trackers (assigned to other cars)
    let editing_car_tracker_id = editing_car.as_ref().and_then(|c| c.tracker_id);
    let assigned_tracker_ids: Vec<i32> = cars::Entity::find()
        .all(db)
        .await
        .map_err(|_| Status::InternalServerError)?
        .into_iter()
        .filter_map(|c| c.tracker_id)
        .filter(|&tid| Some(tid) != editing_car_tracker_id)
        .collect();

    // Fetch available (unassigned) trackers
    let mut query = trackers::Entity::find();
    if !assigned_tracker_ids.is_empty() {
        query = query.filter(trackers::Column::TrackerId.is_not_in(assigned_tracker_ids));
    }
    let available_trackers = query
        .all(db)
        .await
        .map_err(|_| Status::InternalServerError)?;

    // Fetch paginated cars list
    let paginator = cars::Entity::find()
        .find_also_related(trackers::Entity)
        .paginate(db, page_size);

    let raw_total_pages = paginator.num_pages().await.map_err(|_| Status::InternalServerError)?;
    let total_pages = std::cmp::max(1, raw_total_pages);
    let target_page = std::cmp::min(current_page, total_pages);

    let cars_raw = paginator
        .fetch_page(target_page.saturating_sub(1))
        .await
        .map_err(|_| Status::InternalServerError)?;

    let cars = cars_raw
        .into_iter()
        .map(|(c, t)| CarWithTracker { car: c, tracker: t })
        .collect::<Vec<_>>();

    let pages = (1..=total_pages).collect::<Vec<u64>>();

    Ok(CarsTemplate {
        username: user.username.clone(),
        role: user.role.clone(),
        cars,
        available_trackers,
        editing_car,
        current_page: target_page,
        total_pages,
        pages,
        error,
    })
}

#[rocket::get("/crud/cars?<edit>&<page>")]
pub async fn list_cars(
    edit: Option<i32>,
    page: Option<u64>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<CarsTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }
    render_cars(db.inner(), &user, edit, page, None).await
}

#[rocket::post("/crud/cars", data = "<form_data>")]
pub async fn create_car(
    form_data: Form<CarForm<'_>>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<CarsTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let now = chrono::Utc::now().naive_utc();
    let car_type = match form_data.car_type {
        "Delivery" => crate::entities::sea_orm_active_enums::CarType::Delivery,
        _ => crate::entities::sea_orm_active_enums::CarType::Passenger,
    };

    let new_car = cars::ActiveModel {
        name: Set(form_data.name.to_string()),
        police_number: Set(form_data.police_number.to_string()),
        active: Set(1),
        car_type: Set(car_type),
        tracker_id: Set(form_data.tracker_id),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    match new_car.insert(db.inner()).await {
        Ok(_) => render_cars(db.inner(), &user, None, None, None).await,
        Err(err) => render_cars(db.inner(), &user, None, None, Some(err.to_string())).await,
    }
}

#[rocket::put("/crud/cars/<id>?<page>", data = "<form_data>")]
pub async fn update_car(
    id: i32,
    page: Option<u64>,
    form_data: Form<CarForm<'_>>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<CarsTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let car = match cars::Entity::find_by_id(id).one(db.inner()).await {
        Ok(Some(c)) => c,
        Ok(None) => return render_cars(db.inner(), &user, Some(id), page, Some(format!("Car with ID {} not found in the database.", id))).await,
        Err(err) => return render_cars(db.inner(), &user, Some(id), page, Some(err.to_string())).await,
    };

    let car_type = match form_data.car_type {
        "Delivery" => crate::entities::sea_orm_active_enums::CarType::Delivery,
        _ => crate::entities::sea_orm_active_enums::CarType::Passenger,
    };

    let mut active: cars::ActiveModel = car.into();
    active.name = Set(form_data.name.to_string());
    active.police_number = Set(form_data.police_number.to_string());
    active.car_type = Set(car_type);
    active.tracker_id = Set(form_data.tracker_id);
    active.updated_at = Set(chrono::Utc::now().naive_utc());

    match active.update(db.inner()).await {
        Ok(_) => render_cars(db.inner(), &user, None, page, None).await,
        Err(err) => render_cars(db.inner(), &user, Some(id), page, Some(err.to_string())).await,
    }
}

#[rocket::delete("/crud/cars/<id>?<page>")]
pub async fn delete_car(
    id: i32,
    page: Option<u64>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<CarsTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    match cars::Entity::delete_by_id(id).exec(db.inner()).await {
        Ok(_) => render_cars(db.inner(), &user, None, page, None).await,
        Err(err) => render_cars(db.inner(), &user, None, page, Some(err.to_string())).await,
    }
}
