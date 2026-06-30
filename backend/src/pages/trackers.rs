use crate::auth::AuthenticatedUser;
use crate::entities::{trackers, cars};
use askama::Template;
use askama_web::WebTemplate;
use rocket::form::Form;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::State;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

pub struct TrackerWithCar {
    pub tracker: trackers::Model,
    pub car: Option<cars::Model>,
}

#[derive(Template, WebTemplate)]
#[template(path = "trackers.j2")]
pub struct TrackersTemplate {
    pub username: String,
    pub role: String,
    pub trackers: Vec<TrackerWithCar>,
    pub editing_tracker: Option<trackers::Model>,
}

#[derive(rocket::FromForm)]
pub struct TrackerForm<'r> {
    pub name: &'r str,
}

#[rocket::get("/crud/trackers?<edit>")]
pub async fn list_trackers(
    edit: Option<i32>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<TrackersTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let trackers_raw = trackers::Entity::find()
        .find_also_related(crate::entities::cars::Entity)
        .all(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    let trackers = trackers_raw
        .into_iter()
        .map(|(t, c)| TrackerWithCar { tracker: t, car: c })
        .collect::<Vec<_>>();

    let editing_tracker = match edit {
        Some(id) => trackers::Entity::find_by_id(id)
            .one(db.inner())
            .await
            .map_err(|_| Status::InternalServerError)?,
        None => None,
    };

    Ok(TrackersTemplate {
        username: user.username,
        role: user.role,
        trackers,
        editing_tracker,
    })
}

#[rocket::post("/crud/trackers", data = "<form_data>")]
pub async fn create_tracker(
    form_data: Form<TrackerForm<'_>>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Redirect, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let now = chrono::Utc::now().naive_utc();
    let new_tracker = trackers::ActiveModel {
        name: Set(form_data.name.to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    new_tracker
        .insert(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Redirect::to("/crud/trackers"))
}

#[rocket::put("/crud/trackers/<id>", data = "<form_data>")]
pub async fn update_tracker(
    id: i32,
    form_data: Form<TrackerForm<'_>>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Redirect, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let tracker = trackers::Entity::find_by_id(id)
        .one(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    if let Some(t) = tracker {
        let mut active: trackers::ActiveModel = t.into();
        active.name = Set(form_data.name.to_string());
        active.updated_at = Set(chrono::Utc::now().naive_utc());
        active.update(db.inner())
            .await
            .map_err(|_| Status::InternalServerError)?;
    }

    Ok(Redirect::to("/crud/trackers"))
}

#[rocket::delete("/crud/trackers/<id>")]
pub async fn delete_tracker(
    id: i32,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<(), Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    // Hard delete directly from the database using delete_by_id
    trackers::Entity::delete_by_id(id)
        .exec(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(())
}
