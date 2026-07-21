use crate::auth::AuthenticatedUser;
use crate::entities::{cars, trackers};
use askama::Template;
use askama_web::WebTemplate;
use rocket::State;
use rocket::form::Form;
use rocket::http::Status;
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
    pub error: Option<String>,
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
    error: Option<String>,
) -> Result<TrackersTemplate, Status> {
    let current_page = page.unwrap_or(1);
    let page_size = 5;

    let paginator = trackers::Entity::find()
        .find_also_related(crate::entities::cars::Entity)
        .paginate(db, page_size);

    let raw_total_pages = paginator
        .num_pages()
        .await
        .map_err(|_| Status::InternalServerError)?;
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
        error,
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
    render_trackers(db.inner(), &user, edit, page, None).await
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

    match new_tracker.insert(db.inner()).await {
        Ok(_) => render_trackers(db.inner(), &user, None, None, None).await,
        Err(err) => render_trackers(db.inner(), &user, None, None, Some(err.to_string())).await,
    }
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

    let tracker = match trackers::Entity::find_by_id(id).one(db.inner()).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            return render_trackers(
                db.inner(),
                &user,
                Some(id),
                page,
                Some(format!("Tracker with ID {} not found in the database.", id)),
            )
            .await;
        }
        Err(err) => {
            return render_trackers(db.inner(), &user, Some(id), page, Some(err.to_string())).await;
        }
    };

    let mut active: trackers::ActiveModel = tracker.into();
    active.name = Set(form_data.name.to_string());
    active.updated_at = Set(chrono::Utc::now().naive_utc());

    match active.update(db.inner()).await {
        Ok(_) => render_trackers(db.inner(), &user, None, page, None).await,
        Err(err) => render_trackers(db.inner(), &user, Some(id), page, Some(err.to_string())).await,
    }
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

    match trackers::Entity::delete_by_id(id).exec(db.inner()).await {
        Ok(_) => render_trackers(db.inner(), &user, None, page, None).await,
        Err(err) => render_trackers(db.inner(), &user, None, page, Some(err.to_string())).await,
    }
}
