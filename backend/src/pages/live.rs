use crate::auth::AuthenticatedUser;
use crate::entities::{audit, trackers};
use askama::Template;
use askama_web::WebTemplate;
use rocket::http::Status;
use rocket::State;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct TrackerWithLocation {
    pub id: i32,
    pub name: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Template, WebTemplate)]
#[template(path = "live.j2")]
pub struct LiveTemplate {
    pub username: String,
    pub role: String,
    pub trackers: Vec<TrackerWithLocation>,
}

#[rocket::get("/live")]
pub async fn live_tracking(
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<LiveTemplate, Status> {
    let db = db.inner();

    // Fetch all active trackers
    let db_trackers = trackers::Entity::find()
        .filter(trackers::Column::DeletedAt.is_null())
        .order_by_asc(trackers::Column::Name)
        .all(db)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let mut trackers_with_loc = Vec::new();

    for t in db_trackers {
        // Query latest audit record for this tracker
        let latest_audit = audit::Entity::find()
            .filter(audit::Column::TrackerId.eq(t.tracker_id))
            .filter(audit::Column::DeletedAt.is_null())
            .order_by_desc(audit::Column::RecordedAt)
            .one(db)
            .await
            .map_err(|_| Status::InternalServerError)?;

        let (lat, lng) = match latest_audit {
            Some(a) => (Some(a.latitude), Some(a.longitude)),
            None => (None, None),
        };

        trackers_with_loc.push(TrackerWithLocation {
            id: t.tracker_id,
            name: t.name,
            latitude: lat,
            longitude: lng,
        });
    }

    Ok(LiveTemplate {
        username: user.username,
        role: user.role,
        trackers: trackers_with_loc,
    })
}
