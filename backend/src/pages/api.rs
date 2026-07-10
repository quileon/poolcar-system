use crate::auth::AuthenticatedUser;
use crate::entities::sea_orm_active_enums::UserRole;
use crate::entities::users::{self, Entity as Users};
use crate::types::AppConfig;
use askama::Template;
use askama_web::WebTemplate;
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::error::RecvError;
use tracing::{error, info};

#[derive(Serialize)]
pub struct UserResponse {
    pub token: String,
    pub username: String,
    pub role: String,
}

#[derive(Serialize)]
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

#[derive(Deserialize)]
pub struct ApiLoginRequest<'r> {
    pub username: &'r str,
    pub password: &'r str,
}

#[rocket::post("/api/login", data = "<credentials>")]
pub async fn api_login(
    credentials: Json<ApiLoginRequest<'_>>,
    db: &State<DatabaseConnection>,
    config: &State<AppConfig>,
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
                &config.jwt_secret,
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

#[derive(Deserialize, Serialize)]
pub struct GoogleMapPayload {
    #[serde(rename = "textQuery")]
    text_query: String,
    #[serde(rename = "languageCode")]
    language_code: String,
    #[serde(rename = "locationBias")]
    location_bias: Option<GoogleMapLocationBias>,
    #[serde(rename = "pageSize")]
    page_size: Option<u8>,
}

impl GoogleMapPayload {
    pub fn new(
        text_query: String,
        language_code: String,
        latitude: f64,
        longitude: f64,
        radius: f64,
        page_size: Option<u8>,
    ) -> Self {
        Self {
            text_query,
            language_code,
            location_bias: Some(GoogleMapLocationBias {
                circle: GoogleMapLocationBiasCircle {
                    center: PlaceLocation {
                        latitude,
                        longitude,
                    },
                    radius,
                },
            }),
            page_size: Some(page_size.unwrap_or(20)),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct GoogleMapLocationBias {
    circle: GoogleMapLocationBiasCircle,
}

#[derive(Deserialize, Serialize)]
pub struct GoogleMapLocationBiasCircle {
    center: PlaceLocation,
    radius: f64,
}

#[derive(Deserialize, Serialize)]
pub struct GoogleMapResponse {
    #[serde(default)]
    pub places: Vec<Place>,
}

#[derive(Deserialize, Serialize)]
pub struct PlaceLocation {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Deserialize, Serialize)]
pub struct PlaceDisplayName {
    pub text: String,
    #[serde(rename = "languageCode")]
    pub language_code: String,
}

#[derive(Deserialize, Serialize)]
pub struct Place {
    pub id: String,
    #[serde(rename = "formattedAddress")]
    pub formatted_address: String,
    pub location: PlaceLocation,
    #[serde(rename = "displayName")]
    pub display_name: PlaceDisplayName,
}

#[derive(Template, WebTemplate)]
#[template(path = "places_search_results.j2")]
pub struct PlacesSearchResultsTemplate {
    pub places: Vec<Place>,
}

#[rocket::get("/api/places/search?<query>")]
pub async fn search_places(
    query: String,
    config: &State<AppConfig>,
    _user: AuthenticatedUser,
) -> Result<PlacesSearchResultsTemplate, Status> {
    let url = "https://places.googleapis.com/v1/places:searchText";
    let payload = GoogleMapPayload::new(
        query,
        "en".into(),
        -6.370901936057233,
        106.82459298887727,
        50000.0,
        Some(5),
    );

    let client = reqwest::Client::new();
    let response: GoogleMapResponse = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("X-Goog-Api-Key", &config.google_api_key)
        .header(
            "X-Goog-FieldMask",
            "places.id,places.displayName,places.formattedAddress,places.location",
        )
        .json(&payload)
        .send()
        .await
        .map_err(|_| Status::InternalServerError)?
        .json()
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(PlacesSearchResultsTemplate {
        places: response.places,
    })
}

#[rocket::get("/ws/live")]
pub fn live_ws(
    ws: rocket_ws::WebSocket,
    tx: &State<tokio::sync::broadcast::Sender<String>>,
) -> rocket_ws::Channel<'static> {
    use rocket::futures::{SinkExt, StreamExt};
    let mut rx = tx.subscribe();

    ws.channel(move |mut stream| {
        Box::pin(async move {
            loop {
                tokio::select! {
                    msg_res = rx.recv() => {
                        match msg_res {
                            Ok(msg) => {
                                if stream.send(rocket_ws::Message::Text(msg)).await.is_err() {
                                    error!("Failed to send WebSocket message");
                                    break;
                                }
                            }
                            Err(RecvError::Lagged(_)) => {}
                            Err(RecvError::Closed) => {
                                error!("WebSocket connection closed unexpectedly");
                                break;
                            }
                        }
                    }
                    client_msg = stream.next() => {
                        match client_msg {
                            Some(Ok(rocket_ws::Message::Close(_))) | None => {
                                info!("WebSocket connection closed gracefully");
                                break;
                            }
                            Some(Err(_)) => {
                                error!("WebSocket connection error");
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(())
        })
    })
}

// JSON API for creating a car status record (edited by Security / Admin)
#[derive(Deserialize)]
pub struct CreateCarStatusRequest {
    pub car_id: i32,
    pub gas_level: f64,
    pub kilometres: f64,
    pub status_type: String, // "Departure" or "Return"
}

#[derive(Serialize)]
pub struct CreateCarStatusResponse {
    pub car_status_id: i32,
    pub car_id: i32,
    pub gas_level: f64,
    pub kilometres: f64,
    pub status_type: String,
    pub recorded_at: String,
}

#[derive(Serialize)]
pub struct DynamicErrorResponse {
    pub error: String,
}

#[rocket::post("/api/cars/history", data = "<payload>")]
pub async fn api_create_car_status(
    payload: Json<CreateCarStatusRequest>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<CreateCarStatusResponse>, (Status, Json<DynamicErrorResponse>)> {
    use crate::entities::{car_status, cars};
    use sea_orm::{ActiveModelTrait, Set};

    // 1. Authorization: Only Admin and Security are allowed
    if user.role != "Admin" && user.role != "Security" {
        return Err((
            Status::Forbidden,
            Json(DynamicErrorResponse {
                error: "Access denied. Only Admins and Security users can submit status logs."
                    .to_string(),
            }),
        ));
    }

    // 2. Validate that the car exists
    let car_exists = cars::Entity::find_by_id(payload.car_id)
        .one(db.inner())
        .await
        .map_err(|err| {
            (
                Status::InternalServerError,
                Json(DynamicErrorResponse {
                    error: format!("Database error: {}", err),
                }),
            )
        })?;

    if car_exists.is_none() {
        return Err((
            Status::BadRequest,
            Json(DynamicErrorResponse {
                error: format!("Car with ID {} not found", payload.car_id),
            }),
        ));
    }

    // 3. Parse status type
    let status_type = match payload.status_type.as_str() {
        "Departure" => crate::entities::sea_orm_active_enums::StatusType::Departure,
        "Return" => crate::entities::sea_orm_active_enums::StatusType::Return,
        _ => {
            return Err((
                Status::BadRequest,
                Json(DynamicErrorResponse {
                    error: "Invalid status_type. Expected 'Departure' or 'Return'.".to_string(),
                }),
            ));
        }
    };

    // 4. Insert new status log
    let now = chrono::Utc::now().naive_utc();
    let new_status = car_status::ActiveModel {
        car_id: Set(payload.car_id),
        gas_level: Set(payload.gas_level),
        kilometres: Set(payload.kilometres),
        status_type: Set(status_type),
        recorded_at: Set(now),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let inserted = new_status.insert(db.inner()).await.map_err(|err| {
        (
            Status::InternalServerError,
            Json(DynamicErrorResponse {
                error: format!("Failed to insert status: {}", err),
            }),
        )
    })?;

    // 5. Return response
    Ok(Json(CreateCarStatusResponse {
        car_status_id: inserted.car_status_id,
        car_id: inserted.car_id,
        gas_level: inserted.gas_level,
        kilometres: inserted.kilometres,
        status_type: format!("{:?}", inserted.status_type),
        recorded_at: inserted.recorded_at.to_string(),
    }))
}

#[derive(Serialize)]
pub struct CarStatusItem {
    pub car_status_id: i32,
    pub car_id: i32,
    pub car_name: String,
    pub police_number: String,
    pub gas_level: f64,
    pub kilometres: f64,
    pub status_type: String,
    pub recorded_at: String,
}

#[rocket::get("/api/cars/history?<limit>")]
pub async fn api_list_car_status(
    limit: Option<u64>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<CarStatusItem>>, (Status, Json<DynamicErrorResponse>)> {
    use crate::entities::{car_status, cars};
    use sea_orm::{QueryOrder, QuerySelect};

    // 1. Authorization: Only Admin and Security are allowed
    if user.role != "Admin" && user.role != "Security" {
        return Err((
            Status::Forbidden,
            Json(DynamicErrorResponse {
                error: "Access denied. Only Admins and Security users can access status logs."
                    .to_string(),
            }),
        ));
    }

    let limit_val = limit.unwrap_or(20);

    // 2. Fetch logs ordered by recorded_at descending
    let query_result = car_status::Entity::find()
        .filter(car_status::Column::DeletedAt.is_null())
        .find_also_related(cars::Entity)
        .order_by_desc(car_status::Column::RecordedAt)
        .limit(limit_val)
        .all(db.inner())
        .await
        .map_err(|err| {
            (
                Status::InternalServerError,
                Json(DynamicErrorResponse {
                    error: format!("Database error: {}", err),
                }),
            )
        })?;

    let mut response_items = Vec::new();
    for (status, car_opt) in query_result {
        let (car_name, police_number) = if let Some(car) = car_opt {
            (car.name, car.police_number)
        } else {
            ("Unknown Car".to_string(), "Unknown".to_string())
        };

        response_items.push(CarStatusItem {
            car_status_id: status.car_status_id,
            car_id: status.car_id,
            car_name,
            police_number,
            gas_level: status.gas_level,
            kilometres: status.kilometres,
            status_type: format!("{:?}", status.status_type),
            recorded_at: status.recorded_at.to_string(),
        });
    }

    Ok(Json(response_items))
}

#[rocket::get("/api/cars/<id>")]
pub async fn api_get_car(
    id: i32,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<crate::entities::cars::Model>, (Status, Json<DynamicErrorResponse>)> {
    use crate::entities::cars;

    // 1. Authorization: Only Admin and Security are allowed
    if user.role != "Admin" && user.role != "Security" {
        return Err((
            Status::Forbidden,
            Json(DynamicErrorResponse {
                error: "Access denied. Only Admins and Security users can access vehicle details."
                    .to_string(),
            }),
        ));
    }

    // 2. Fetch the car details
    let car = cars::Entity::find_by_id(id)
        .filter(cars::Column::DeletedAt.is_null())
        .one(db.inner())
        .await
        .map_err(|err| {
            (
                Status::InternalServerError,
                Json(DynamicErrorResponse {
                    error: format!("Database error: {}", err),
                }),
            )
        })?;

    match car {
        Some(c) => Ok(Json(c)),
        None => Err((
            Status::NotFound,
            Json(DynamicErrorResponse {
                error: format!("Car with ID {} not found", id),
            }),
        )),
    }
}
