use crate::auth::AuthenticatedUser;
use crate::entities::{trackers, cars};
use askama::Template;
use askama_web::WebTemplate;
use rocket::form::Form;
use rocket::http::Status;
use rocket::State;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, Set};

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
    pub current_page: u64,
    pub total_pages: u64,
    pub pages: Vec<u64>,
}

#[derive(rocket::FromForm)]
pub struct TrackerForm<'r> {
    pub name: &'r str,
}

async fn render_trackers(
    db: &DatabaseConnection,
    user: &AuthenticatedUser,
    edit: Option<i32>,
    page: Option<u64>,
) -> Result<TrackersTemplate, Status> {
    let current_page = page.unwrap_or(1);
    let page_size = 5;

    let paginator = trackers::Entity::find()
        .find_also_related(crate::entities::cars::Entity)
        .paginate(db, page_size);

    let raw_total_pages = paginator.num_pages().await.map_err(|_| Status::InternalServerError)?;
    let total_pages = std::cmp::max(1, raw_total_pages);
    let target_page = std::cmp::min(current_page, total_pages);

    let trackers_raw = paginator
        .fetch_page(target_page.saturating_sub(1))
        .await
        .map_err(|_| Status::InternalServerError)?;

    let trackers = trackers_raw
        .into_iter()
        .map(|(t, c)| TrackerWithCar { tracker: t, car: c })
        .collect::<Vec<_>>();

    let editing_tracker = match edit {
        Some(id) => trackers::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|_| Status::InternalServerError)?,
        None => None,
    };

    let pages = (1..=total_pages).collect::<Vec<u64>>();

    Ok(TrackersTemplate {
        username: user.username.clone(),
        role: user.role.clone(),
        trackers,
        editing_tracker,
        current_page: target_page,
        total_pages,
        pages,
    })
}

#[rocket::get("/crud/trackers?<edit>&<page>")]
pub async fn list_trackers(
    edit: Option<i32>,
    page: Option<u64>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<TrackersTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }
    render_trackers(db.inner(), &user, edit, page).await
}

#[rocket::post("/crud/trackers", data = "<form_data>")]
pub async fn create_tracker(
    form_data: Form<TrackerForm<'_>>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<TrackersTemplate, Status> {
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

    render_trackers(db.inner(), &user, None, None).await
}

#[rocket::put("/crud/trackers/<id>?<page>", data = "<form_data>")]
pub async fn update_tracker(
    id: i32,
    page: Option<u64>,
    form_data: Form<TrackerForm<'_>>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<TrackersTemplate, Status> {
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

    render_trackers(db.inner(), &user, None, page).await
}

#[rocket::delete("/crud/trackers/<id>?<page>")]
pub async fn delete_tracker(
    id: i32,
    page: Option<u64>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<TrackersTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    trackers::Entity::delete_by_id(id)
        .exec(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    render_trackers(db.inner(), &user, None, page).await
}
