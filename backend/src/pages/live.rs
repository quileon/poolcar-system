use crate::auth::AuthenticatedUser;
use crate::entities::trackers;
use crate::loops::mqtt::MqttPayload;
use askama::Template;
use askama_web::WebTemplate;
use redis::AsyncCommands;
use rocket::State;
use rocket::http::Status;
use sea_orm::{DatabaseConnection, EntityTrait};

#[derive(Template, WebTemplate)]
#[template(path = "live.j2")]
pub struct LiveTemplate {
    pub username: String,
    pub role: String,
    pub trackers: Vec<trackers::Model>,
    pub tracker_payloads: Vec<MqttPayload>,
    pub error: Option<String>,
}

async fn render_live(
    db: &DatabaseConnection,
    redis: &redis::Client,
    user: &AuthenticatedUser,
    error: Option<String>,
) -> Result<LiveTemplate, Status> {
    let mut conn = match redis.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(err) => {
            return Ok(LiveTemplate {
                username: user.username.clone(),
                role: user.role.clone(),
                trackers: vec![],
                tracker_payloads: vec![],
                error: Some(err.to_string()),
            });
        }
    };

    let trackers = match trackers::Entity::find().all(db).await {
        Ok(t) => t,
        Err(err) => {
            return Ok(LiveTemplate {
                username: user.username.clone(),
                role: user.role.clone(),
                trackers: vec![],
                tracker_payloads: vec![],
                error: Some(err.to_string()),
            });
        }
    };

    let mut tracker_payloads: Vec<MqttPayload> = Vec::new();
    for tracker in &trackers {
        let bytes: Option<Vec<u8>> = match conn
            .get(format!("tracker:{}:live", tracker.tracker_id))
            .await
        {
            Ok(b) => b,
            Err(err) => {
                return Ok(LiveTemplate {
                    username: user.username.clone(),
                    role: user.role.clone(),
                    trackers,
                    tracker_payloads,
                    error: Some(err.to_string()),
                });
            }
        };
        if let Some(bytes) = bytes {
            if let Ok(payload) = serde_json::from_slice::<MqttPayload>(&bytes) {
                tracker_payloads.push(payload);
            }
        }
    }

    Ok(LiveTemplate {
        username: user.username.clone(),
        role: user.role.clone(),
        trackers,
        tracker_payloads,
        error,
    })
}

#[rocket::get("/live")]
pub async fn live_tracking(
    db: &State<DatabaseConnection>,
    redis: &State<redis::Client>,
    user: AuthenticatedUser,
) -> Result<LiveTemplate, Status> {
    if user.role == "Security" {
        return Err(Status::Forbidden);
    }
    render_live(db.inner(), redis.inner(), &user, None).await
}
