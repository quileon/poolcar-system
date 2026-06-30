use crate::auth::{AuthenticatedUser, JwtSecret};
use crate::entities::sea_orm_active_enums::UserRole;
use crate::entities::users::{self, Entity as Users};
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

#[derive(serde::Serialize)]
pub struct UserResponse {
    pub token: String,
    pub username: String,
    pub role: String,
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: &'static str,
}

#[rocket::get("/api/verify")]
pub fn verify(
    user: Option<AuthenticatedUser>,
) -> Result<Json<UserResponse>, (Status, Json<ErrorResponse>)> {
    match user {
        Some(u) => {
            if u.role == "Employee" {
                return Err((
                    Status::Forbidden,
                    Json(ErrorResponse {
                        error: "Employee users cannot log in",
                    }),
                ));
            }

            Ok(Json(UserResponse {
                token: u.token,
                username: u.username,
                role: u.role,
            }))
        }
        None => Err((
            Status::Unauthorized,
            Json(ErrorResponse {
                error: "Session expired or invalid",
            }),
        )),
    }
}

#[derive(serde::Deserialize)]
pub struct ApiLoginRequest<'r> {
    pub username: &'r str,
    pub password: &'r str,
}

#[rocket::post("/api/login", data = "<credentials>")]
pub async fn api_login(
    credentials: Json<ApiLoginRequest<'_>>,
    db: &State<DatabaseConnection>,
    jwt_secret: &State<JwtSecret>,
) -> Result<Json<UserResponse>, (Status, Json<ErrorResponse>)> {
    let username = credentials.username;
    let password = credentials.password;

    let user = Users::find()
        .filter(users::Column::Username.eq(username))
        .one(db.inner())
        .await
        .map_err(|_| {
            (
                Status::InternalServerError,
                Json(ErrorResponse {
                    error: "Database error",
                }),
            )
        })?;

    if let Some(user) = user {
        if user.user_role == UserRole::Employee {
            return Err((
                Status::Forbidden,
                Json(ErrorResponse {
                    error: "Employee users cannot log in",
                }),
            ));
        }
        if user.password == password {
            let token = crate::auth::create_token(
                &user.username,
                &format!("{:?}", user.user_role),
                &jwt_secret.0,
            )
            .map_err(|_| {
                (
                    Status::InternalServerError,
                    Json(ErrorResponse {
                        error: "Token generation failed",
                    }),
                )
            })?;

            return Ok(Json(UserResponse {
                token,
                username: user.username,
                role: format!("{:?}", user.user_role),
            }));
        }
    }

    Err((
        Status::Unauthorized,
        Json(ErrorResponse {
            error: "Invalid username or password",
        }),
    ))
}
