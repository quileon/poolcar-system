use crate::auth::AuthenticatedUser;
use askama::Template;
use askama_web::WebTemplate;

#[derive(Template, WebTemplate)]
#[template(path = "dashboard.j2")]
pub struct DashboardTemplate {
    pub error: Option<String>,
    pub username: String,
    pub role: String,
}

#[rocket::get("/")]
pub fn dashboard(user: AuthenticatedUser) -> DashboardTemplate {
    // Rocket automatically guarantees the user is authenticated here!
    DashboardTemplate {
        error: None,
        username: user.username,
        role: user.role,
    }
}
