use crate::auth::AuthenticatedUser;
use crate::entities::{cars, trackers};
use askama::Template;
use askama_web::WebTemplate;
use rocket::form::Form;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::{FromForm, State};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

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
}

#[derive(FromForm)]
pub struct CarForm<'r> {
    pub name: &'r str,
    pub police_number: &'r str,
    pub car_type: &'r str,
    pub tracker_id: Option<i32>,
}

#[rocket::get("/crud/cars?<edit>")]
pub async fn list_cars(
    edit: Option<i32>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<CarsTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let cars_raw = cars::Entity::find()
        .find_also_related(trackers::Entity)
        .all(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    let cars = cars_raw
        .into_iter()
        .map(|(c, t)| CarWithTracker { car: c, tracker: t })
        .collect::<Vec<_>>();

    let editing_car = match edit {
        Some(id) => cars::Entity::find_by_id(id)
            .one(db.inner())
            .await
            .map_err(|_| Status::InternalServerError)?,
        None => None,
    };

    let editing_car_tracker_id = editing_car.as_ref().and_then(|c| c.tracker_id);
    let assigned_tracker_ids: Vec<i32> = cars::Entity::find()
        .all(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?
        .into_iter()
        .filter_map(|c| c.tracker_id)
        .filter(|&tid| Some(tid) != editing_car_tracker_id)
        .collect();

    let mut query = trackers::Entity::find();
    if !assigned_tracker_ids.is_empty() {
        query = query.filter(trackers::Column::TrackerId.is_not_in(assigned_tracker_ids));
    }
    let available_trackers = query
        .all(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(CarsTemplate {
        username: user.username,
        role: user.role,
        cars,
        available_trackers,
        editing_car,
    })
}

#[rocket::post("/crud/cars", data = "<form_data>")]
pub async fn create_car(
    form_data: Form<CarForm<'_>>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Redirect, Status> {
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

    new_car
        .insert(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Redirect::to("/crud/cars"))
}

#[rocket::put("/crud/cars/<id>", data = "<form_data>")]
pub async fn update_car(
    id: i32,
    form_data: Form<CarForm<'_>>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Redirect, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let car = cars::Entity::find_by_id(id)
        .one(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    if let Some(c) = car {
        let car_type = match form_data.car_type {
            "Delivery" => crate::entities::sea_orm_active_enums::CarType::Delivery,
            _ => crate::entities::sea_orm_active_enums::CarType::Passenger,
        };

        let mut active: cars::ActiveModel = c.into();
        active.name = Set(form_data.name.to_string());
        active.police_number = Set(form_data.police_number.to_string());
        active.car_type = Set(car_type);
        active.tracker_id = Set(form_data.tracker_id);
        active.updated_at = Set(chrono::Utc::now().naive_utc());
        active
            .update(db.inner())
            .await
            .map_err(|_| Status::InternalServerError)?;
    }

    Ok(Redirect::to("/crud/cars"))
}

#[rocket::delete("/crud/cars/<id>")]
pub async fn delete_car(
    id: i32,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Redirect, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    cars::Entity::delete_by_id(id)
        .exec(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Redirect::to("/crud/cars"))
}
