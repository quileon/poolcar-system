use crate::auth::AuthenticatedUser;
use crate::entities::{car_status, cars};
use askama::Template;
use askama_web::WebTemplate;
use chrono::NaiveDateTime;
use rocket::form::Form;
use rocket::http::Status;
use rocket::{FromForm, State};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder, Set};

pub struct StatusWithCar {
    pub status: car_status::Model,
    pub car: Option<cars::Model>,
}

#[derive(Template, WebTemplate)]
#[template(path = "car_history.j2")]
pub struct CarHistoryTemplate {
    pub username: String,
    pub role: String,
    pub history: Vec<StatusWithCar>,
    pub cars: Vec<cars::Model>,
    pub editing_status: Option<car_status::Model>,
    pub current_page: u64,
    pub total_pages: u64,
    pub pages: Vec<u64>,
    pub error: Option<String>,
}

#[derive(FromForm)]
pub struct CarStatusForm<'r> {
    pub car_id: i32,
    pub gas_level: f64,
    pub kilometres: f64,
    pub status_type: &'r str,
    pub recorded_at: &'r str,
}

async fn render_history(
    db: &DatabaseConnection,
    user: &AuthenticatedUser,
    edit: Option<i32>,
    page: Option<u64>,
    error: Option<String>,
) -> Result<CarHistoryTemplate, Status> {
    let current_page = page.unwrap_or(1);
    let page_size = 5;

    let editing_status = match edit {
        Some(id) => car_status::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|_| Status::InternalServerError)?,
        None => None,
    };

    let cars = cars::Entity::find()
        .all(db)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let paginator = car_status::Entity::find()
        .find_also_related(cars::Entity)
        .order_by_desc(car_status::Column::RecordedAt)
        .paginate(db, page_size);

    let raw_total_pages = paginator
        .num_pages()
        .await
        .map_err(|_| Status::InternalServerError)?;
    let total_pages = std::cmp::max(1, raw_total_pages);
    let target_page = std::cmp::min(current_page, total_pages);

    let history_raw = paginator
        .fetch_page(target_page.saturating_sub(1))
        .await
        .map_err(|_| Status::InternalServerError)?;

    let history = history_raw
        .into_iter()
        .map(|(s, c)| StatusWithCar { status: s, car: c })
        .collect::<Vec<_>>();

    let pages = (1..=total_pages).collect::<Vec<u64>>();

    Ok(CarHistoryTemplate {
        username: user.username.clone(),
        role: user.role.clone(),
        history,
        cars,
        editing_status,
        current_page: target_page,
        total_pages,
        pages,
        error,
    })
}

#[rocket::get("/crud/cars/history?<edit>&<page>")]
pub async fn list_history(
    edit: Option<i32>,
    page: Option<u64>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<CarHistoryTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }
    render_history(db.inner(), &user, edit, page, None).await
}

#[rocket::post("/crud/cars/history", data = "<form_data>")]
pub async fn create_history(
    form_data: Form<CarStatusForm<'_>>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<CarHistoryTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let recorded_at = match NaiveDateTime::parse_from_str(form_data.recorded_at, "%Y-%m-%dT%H:%M") {
        Ok(dt) => dt,
        Err(_) => {
            return render_history(
                db.inner(),
                &user,
                None,
                None,
                Some("Invalid Recorded At date/time format.".to_string()),
            )
            .await;
        }
    };

    let status_type = match form_data.status_type {
        "Departure" => crate::entities::sea_orm_active_enums::StatusType::Departure,
        _ => crate::entities::sea_orm_active_enums::StatusType::Return,
    };

    let now = chrono::Utc::now().naive_utc();

    let new_status = car_status::ActiveModel {
        car_id: Set(form_data.car_id),
        gas_level: Set(form_data.gas_level),
        kilometres: Set(form_data.kilometres),
        status_type: Set(status_type),
        recorded_at: Set(recorded_at),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    match new_status.insert(db.inner()).await {
        Ok(_) => render_history(db.inner(), &user, None, None, None).await,
        Err(err) => render_history(db.inner(), &user, None, None, Some(err.to_string())).await,
    }
}

#[rocket::put("/crud/cars/history/<id>?<page>", data = "<form_data>")]
pub async fn update_history(
    id: i32,
    page: Option<u64>,
    form_data: Form<CarStatusForm<'_>>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<CarHistoryTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let status_record = match car_status::Entity::find_by_id(id).one(db.inner()).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return render_history(
                db.inner(),
                &user,
                Some(id),
                page,
                Some(format!("Status record with ID {} not found.", id)),
            )
            .await;
        }
        Err(err) => {
            return render_history(db.inner(), &user, Some(id), page, Some(err.to_string())).await;
        }
    };

    let recorded_at = match NaiveDateTime::parse_from_str(form_data.recorded_at, "%Y-%m-%dT%H:%M") {
        Ok(dt) => dt,
        Err(_) => {
            return render_history(
                db.inner(),
                &user,
                Some(id),
                page,
                Some("Invalid Recorded At date/time format.".to_string()),
            )
            .await;
        }
    };

    let status_type = match form_data.status_type {
        "Departure" => crate::entities::sea_orm_active_enums::StatusType::Departure,
        _ => crate::entities::sea_orm_active_enums::StatusType::Return,
    };

    let mut active: car_status::ActiveModel = status_record.into();
    active.car_id = Set(form_data.car_id);
    active.gas_level = Set(form_data.gas_level);
    active.kilometres = Set(form_data.kilometres);
    active.status_type = Set(status_type);
    active.recorded_at = Set(recorded_at);
    active.updated_at = Set(chrono::Utc::now().naive_utc());

    match active.update(db.inner()).await {
        Ok(_) => render_history(db.inner(), &user, None, page, None).await,
        Err(err) => render_history(db.inner(), &user, Some(id), page, Some(err.to_string())).await,
    }
}

#[rocket::delete("/crud/cars/history/<id>?<page>")]
pub async fn delete_history(
    id: i32,
    page: Option<u64>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<CarHistoryTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    match car_status::Entity::delete_by_id(id).exec(db.inner()).await {
        Ok(_) => render_history(db.inner(), &user, None, page, None).await,
        Err(err) => render_history(db.inner(), &user, None, page, Some(err.to_string())).await,
    }
}
