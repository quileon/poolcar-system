use crate::auth::{self, JwtSecret};
use crate::entities::sea_orm_active_enums::UserRole;
use crate::entities::users::{self, Entity as Users};
use crate::types::HxRedirect;
use askama::Template;
use askama_web::WebTemplate;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar, Header};
use rocket::response::Redirect;
use rocket::{FromForm, State};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

#[derive(Template, WebTemplate)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub error: Option<String>,
    pub username: Option<String>,
}

#[derive(FromForm)]
pub struct LoginForm<'r> {
    pub username: &'r str,
    pub password: &'r str,
}

#[rocket::get("/login?<error>")]
pub fn login(error: Option<String>) -> LoginTemplate {
    LoginTemplate {
        error,
        username: None,
    }
}

#[rocket::catch(401)]
pub fn unauthorized() -> Redirect {
    Redirect::to("/login?error=Session+expired")
}

#[rocket::post("/login", data = "<form_data>")]
pub async fn post_login<'r>(
    form_data: Form<LoginForm<'r>>,
    db: &State<DatabaseConnection>,
    jwt_secret: &State<JwtSecret>,
    cookies: &CookieJar<'_>,
) -> Result<HxRedirect, LoginTemplate> {
    let username = form_data.username;
    let password = form_data.password;

    let user: Option<users::Model> = Users::find()
        .filter(users::Column::Username.eq(username))
        .one(db.inner())
        .await
        .map_err(|_| LoginTemplate {
            error: Some("Database error".to_owned()),
            username: Some(username.to_owned()),
        })?;

    if let Some(user) = user {
        if user.user_role == UserRole::Employee {
            return Err(LoginTemplate {
                error: Some("Employee users cannot log in".to_owned()),
                username: Some(username.to_owned()),
            });
        }
        if user.password == password {
            let token = auth::create_token(
                &user.username,
                &format!("{:?}", user.user_role),
                &jwt_secret.0,
            )
            .map_err(|_| LoginTemplate {
                error: Some("Token generation failed".to_owned()),
                username: Some(username.to_owned()),
            })?;

            cookies.add(Cookie::new("session_token", token));

            return Ok(HxRedirect {
                body: "",
                header: Header::new("HX-Redirect", "/"),
            });
        }
    }

    Err(LoginTemplate {
        error: Some("Invalid username or password".to_owned()),
        username: Some(username.to_owned()),
    })
}

#[rocket::get("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> HxRedirect {
    cookies.remove(Cookie::from("session_token"));
    HxRedirect {
        body: "",
        header: Header::new("HX-Redirect", "/login"),
    }
}
