use crate::auth::AuthenticatedUser;
use crate::entities::{cars, trackers};
use askama::Template;
use askama_web::WebTemplate;
use rocket::State;
use rocket::http::Status;
use sea_orm::{DatabaseConnection, EntityTrait};

#[derive(Template, WebTemplate)]
#[template(path = "audit.j2")]
pub struct AuditTemplate {
    pub username: String,
    pub role: String,
    pub trackers: Vec<trackers::Model>,
    pub cars: Vec<cars::Model>,
    pub error: Option<String>,
}

#[rocket::get("/audit")]
pub async fn audit_page(
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<AuditTemplate, Status> {
    if user.role == "Security" {
        return Err(Status::Forbidden);
    }

    let trackers = match trackers::Entity::find().all(db.inner()).await {
        Ok(t) => t,
        Err(err) => {
            return Ok(AuditTemplate {
                username: user.username.clone(),
                role: user.role.clone(),
                trackers: vec![],
                cars: vec![],
                error: Some(err.to_string()),
            });
        }
    };

    let cars = match cars::Entity::find().all(db.inner()).await {
        Ok(c) => c,
        Err(err) => {
            return Ok(AuditTemplate {
                username: user.username.clone(),
                role: user.role.clone(),
                trackers,
                cars: vec![],
                error: Some(err.to_string()),
            });
        }
    };

    Ok(AuditTemplate {
        username: user.username,
        role: user.role,
        trackers,
        cars,
        error: None,
    })
}
