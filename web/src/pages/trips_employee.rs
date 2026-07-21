use crate::auth::AuthenticatedUser;
use crate::entities::{activities as trips_entity, cars, contacts, trackers};
use crate::pages::trips::{
    parse_datetime, reload_active_trips_cache, TripForm, TripWithDetails,
};
use askama::Template;
use askama_web::WebTemplate;
use rocket::form::Form;
use rocket::http::Status;
use rocket::State;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use tracing::error;

#[derive(Template, WebTemplate)]
#[template(path = "trips_employee.j2")]
pub struct TripsEmployeeTemplate {
    pub username: String,
    pub role: String,
    pub trips: Vec<TripWithDetails>,
    pub contacts: Vec<contacts::Model>,
    pub editing_trip: Option<trips_entity::Model>,
    pub current_page: u64,
    pub total_pages: u64,
    pub pages: Vec<u64>,
    pub error: Option<String>,
}

async fn render_trips_employee(
    db: &DatabaseConnection,
    user: &AuthenticatedUser,
    edit: Option<i32>,
    page: Option<u64>,
    error: Option<String>,
) -> Result<TripsEmployeeTemplate, Status> {
    let current_page = page.unwrap_or(1);
    let page_size = 5;

    let editing_trip = match edit {
        Some(id) => trips_entity::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|_| Status::InternalServerError)?,
        None => None,
    };

    let all_contacts = contacts::Entity::find()
        .order_by_asc(contacts::Column::Name)
        .all(db)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let paginator = trips_entity::Entity::find()
        .order_by_desc(trips_entity::Column::CreatedAt)
        .paginate(db, page_size);

    let raw_total_pages = paginator
        .num_pages()
        .await
        .map_err(|_| Status::InternalServerError)?;
    let total_pages = std::cmp::max(1, raw_total_pages);
    let target_page = std::cmp::min(current_page, total_pages);

    let raw_trips = paginator
        .fetch_page(target_page.saturating_sub(1))
        .await
        .map_err(|_| Status::InternalServerError)?;

    let car_ids: Vec<i32> = raw_trips.iter().filter_map(|t| t.car_id).collect();
    let contact_ids: Vec<i32> = raw_trips.iter().map(|t| t.contact_id).collect();
    let tracker_ids: Vec<i32> = raw_trips.iter().filter_map(|t| t.tracker_id).collect();

    let related_cars = if !car_ids.is_empty() {
        cars::Entity::find()
            .filter(cars::Column::CarId.is_in(car_ids))
            .all(db)
            .await
            .map_err(|_| Status::InternalServerError)?
    } else {
        vec![]
    };

    let related_contacts = if !contact_ids.is_empty() {
        contacts::Entity::find()
            .filter(contacts::Column::ContactId.is_in(contact_ids))
            .all(db)
            .await
            .map_err(|_| Status::InternalServerError)?
    } else {
        vec![]
    };

    let related_trackers = if !tracker_ids.is_empty() {
        trackers::Entity::find()
            .filter(trackers::Column::TrackerId.is_in(tracker_ids))
            .all(db)
            .await
            .map_err(|_| Status::InternalServerError)?
    } else {
        vec![]
    };

    let trips = raw_trips
        .into_iter()
        .map(|t| {
            let car = t
                .car_id
                .and_then(|cid| related_cars.iter().find(|c| c.car_id == cid).cloned());
            let contact = related_contacts
                .iter()
                .find(|c| c.contact_id == t.contact_id)
                .cloned();
            let tracker = t.tracker_id.and_then(|tid| {
                related_trackers
                    .iter()
                    .find(|tr| tr.tracker_id == tid)
                    .cloned()
            });
            TripWithDetails {
                trip: t,
                car,
                contact,
                tracker,
            }
        })
        .collect::<Vec<_>>();

    let pages = (1..=total_pages).collect::<Vec<u64>>();

    Ok(TripsEmployeeTemplate {
        username: user.username.clone(),
        role: user.role.clone(),
        trips,
        contacts: all_contacts,
        editing_trip,
        current_page: target_page,
        total_pages,
        pages,
        error,
    })
}

#[rocket::get("/trips?<edit>&<page>")]
pub async fn list_trips_employee(
    edit: Option<i32>,
    page: Option<u64>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<TripsEmployeeTemplate, Status> {
    render_trips_employee(db.inner(), &user, edit, page, None).await
}

#[rocket::post("/trips", data = "<form_data>")]
pub async fn create_trip_employee(
    form_data: Form<TripForm<'_>>,
    db: &State<DatabaseConnection>,
    redis: &State<redis::Client>,
    tx: &State<tokio::sync::broadcast::Sender<String>>,
    user: AuthenticatedUser,
) -> Result<TripsEmployeeTemplate, Status> {
    let started_at = parse_datetime(form_data.started_at);
    let finished_at = parse_datetime(form_data.finished_at);

    let activity_type = match form_data.activity_type {
        "Delivery" => crate::entities::sea_orm_active_enums::ActivityType::Delivery,
        "Meeting" => crate::entities::sea_orm_active_enums::ActivityType::Meeting,
        _ => crate::entities::sea_orm_active_enums::ActivityType::TrialT1,
    };

    let now = chrono::Utc::now().naive_utc();

    let new_trip = trips_entity::ActiveModel {
        car_id: Set(None),
        contact_id: Set(form_data.contact_id),
        activity_type: Set(activity_type),
        tracker_id: Set(None),
        started_at: Set(started_at),
        finished_at: Set(None),
        finished_latitude: Set(None),
        finished_longitude: Set(None),
        description: Set(form_data.description.map(|s| s.to_string())),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    match new_trip.insert(db.inner()).await {
        Ok(inserted) => {
            if let Err(e) = reload_active_trips_cache(db.inner(), redis.inner()).await {
                error!("Failed to reload active trips cache: {:?}", e);
            }
            if finished_at.is_some() {
                let ws_message = serde_json::json!({
                    "message_type": "remove_destination",
                    "data": {
                        "activity_id": inserted.activity_id,
                    }
                });
                let _ = tx.send(ws_message.to_string());
            } else {
                if let Ok(Some(contact)) = contacts::Entity::find_by_id(inserted.contact_id).one(db.inner()).await {
                    let ws_message = serde_json::json!({
                        "message_type": "update_destination",
                        "data": {
                            "activity_id": inserted.activity_id,
                            "contact_name": contact.name,
                            "contact_latitude": contact.latitude,
                            "contact_longitude": contact.longitude,
                        }
                    });
                    let _ = tx.send(ws_message.to_string());
                }
            }
            render_trips_employee(db.inner(), &user, None, None, None).await
        }
        Err(err) => render_trips_employee(db.inner(), &user, None, None, Some(err.to_string())).await,
    }
}

#[rocket::put("/trips/<id>?<page>", data = "<form_data>")]
pub async fn update_trip_employee(
    id: i32,
    page: Option<u64>,
    form_data: Form<TripForm<'_>>,
    db: &State<DatabaseConnection>,
    redis: &State<redis::Client>,
    tx: &State<tokio::sync::broadcast::Sender<String>>,
    user: AuthenticatedUser,
) -> Result<TripsEmployeeTemplate, Status> {
    let trip_record = match trips_entity::Entity::find_by_id(id).one(db.inner()).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            return render_trips_employee(
                db.inner(),
                &user,
                Some(id),
                page,
                Some(format!("Trip with ID {} not found.", id)),
            )
            .await;
        }
        Err(err) => {
            return render_trips_employee(db.inner(), &user, Some(id), page, Some(err.to_string())).await;
        }
    };

    let started_at = parse_datetime(form_data.started_at);
    let activity_type = match form_data.activity_type {
        "Delivery" => crate::entities::sea_orm_active_enums::ActivityType::Delivery,
        "Meeting" => crate::entities::sea_orm_active_enums::ActivityType::Meeting,
        _ => crate::entities::sea_orm_active_enums::ActivityType::TrialT1,
    };

    let mut active: trips_entity::ActiveModel = trip_record.clone().into();
    active.contact_id = Set(form_data.contact_id);
    active.activity_type = Set(activity_type);
    active.started_at = Set(started_at);
    active.description = Set(form_data.description.map(|s| s.to_string()));
    active.updated_at = Set(chrono::Utc::now().naive_utc());

    match active.update(db.inner()).await {
        Ok(_) => {
            if let Err(e) = reload_active_trips_cache(db.inner(), redis.inner()).await {
                error!("Failed to reload active trips cache: {:?}", e);
            }
            if trip_record.finished_at.is_some() {
                let ws_message = serde_json::json!({
                    "message_type": "remove_destination",
                    "data": {
                        "activity_id": id,
                    }
                });
                let _ = tx.send(ws_message.to_string());
            } else {
                if let Ok(Some(contact)) = contacts::Entity::find_by_id(form_data.contact_id).one(db.inner()).await {
                    let ws_message = serde_json::json!({
                        "message_type": "update_destination",
                        "data": {
                            "activity_id": id,
                            "contact_name": contact.name,
                            "contact_latitude": contact.latitude,
                            "contact_longitude": contact.longitude,
                        }
                    });
                    let _ = tx.send(ws_message.to_string());
                }
            }
            render_trips_employee(db.inner(), &user, None, page, None).await
        }
        Err(err) => render_trips_employee(db.inner(), &user, Some(id), page, Some(err.to_string())).await,
    }
}

#[rocket::delete("/trips/<id>?<page>")]
pub async fn delete_trip_employee(
    id: i32,
    page: Option<u64>,
    db: &State<DatabaseConnection>,
    redis: &State<redis::Client>,
    tx: &State<tokio::sync::broadcast::Sender<String>>,
    user: AuthenticatedUser,
) -> Result<TripsEmployeeTemplate, Status> {
    match trips_entity::Entity::delete_by_id(id)
        .exec(db.inner())
        .await
    {
        Ok(_) => {
            if let Err(e) = reload_active_trips_cache(db.inner(), redis.inner()).await {
                error!("Failed to reload active trips cache: {:?}", e);
            }
            let ws_message = serde_json::json!({
                "message_type": "remove_destination",
                "data": {
                    "activity_id": id,
                }
            });
            let _ = tx.send(ws_message.to_string());
            render_trips_employee(db.inner(), &user, None, page, None).await
        }
        Err(err) => render_trips_employee(db.inner(), &user, None, page, Some(err.to_string())).await,
    }
}

#[rocket::post("/trips/<id>/finish?<page>")]
pub async fn finish_trip_employee(
    id: i32,
    page: Option<u64>,
    db: &State<DatabaseConnection>,
    redis: &State<redis::Client>,
    tx: &State<tokio::sync::broadcast::Sender<String>>,
    user: AuthenticatedUser,
) -> Result<TripsEmployeeTemplate, Status> {
    let trip_record = match trips_entity::Entity::find_by_id(id).one(db.inner()).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            return render_trips_employee(
                db.inner(),
                &user,
                Some(id),
                page,
                Some(format!("Trip with ID {} not found.", id)),
            )
            .await;
        }
        Err(err) => {
            return render_trips_employee(db.inner(), &user, Some(id), page, Some(err.to_string())).await;
        }
    };

    let now = chrono::Utc::now().naive_utc();

    let mut active: trips_entity::ActiveModel = trip_record.into();
    active.finished_at = Set(Some(now));
    active.updated_at = Set(now);

    match active.update(db.inner()).await {
        Ok(_) => {
            if let Err(e) = reload_active_trips_cache(db.inner(), redis.inner()).await {
                error!("Failed to reload active trips cache: {:?}", e);
            }
            let ws_message = serde_json::json!({
                "message_type": "remove_destination",
                "data": {
                    "activity_id": id,
                }
            });
            let _ = tx.send(ws_message.to_string());

            render_trips_employee(db.inner(), &user, None, page, None).await
        }
        Err(err) => render_trips_employee(db.inner(), &user, Some(id), page, Some(err.to_string())).await,
    }
}

pub struct CsvResponse(pub Vec<u8>);

impl<'r> rocket::response::Responder<'r, 'static> for CsvResponse {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        use rocket::http::{ContentType, Header};
        rocket::response::Response::build()
            .header(ContentType::CSV)
            .header(Header::new("Content-Disposition", "attachment; filename=\"trips_export.csv\""))
            .sized_body(self.0.len(), std::io::Cursor::new(self.0))
            .ok()
    }
}

#[rocket::get("/trips/export")]
pub async fn export_trips_csv(
    db: &State<DatabaseConnection>,
    _user: AuthenticatedUser,
) -> Result<CsvResponse, Status> {
    let raw_trips = trips_entity::Entity::find()
        .order_by_desc(trips_entity::Column::CreatedAt)
        .all(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    let car_ids: Vec<i32> = raw_trips.iter().filter_map(|t| t.car_id).collect();
    let contact_ids: Vec<i32> = raw_trips.iter().map(|t| t.contact_id).collect();
    let tracker_ids: Vec<i32> = raw_trips.iter().filter_map(|t| t.tracker_id).collect();

    let related_cars = if !car_ids.is_empty() {
        cars::Entity::find()
            .filter(cars::Column::CarId.is_in(car_ids))
            .all(db.inner())
            .await
            .map_err(|_| Status::InternalServerError)?
    } else {
        vec![]
    };

    let related_contacts = if !contact_ids.is_empty() {
        contacts::Entity::find()
            .filter(contacts::Column::ContactId.is_in(contact_ids))
            .all(db.inner())
            .await
            .map_err(|_| Status::InternalServerError)?
    } else {
        vec![]
    };

    let related_trackers = if !tracker_ids.is_empty() {
        trackers::Entity::find()
            .filter(trackers::Column::TrackerId.is_in(tracker_ids))
            .all(db.inner())
            .await
            .map_err(|_| Status::InternalServerError)?
    } else {
        vec![]
    };

    let mut wtr = csv::Writer::from_writer(Vec::new());

    wtr.write_record(&[
        "activity_id",
        "activity_type",
        "description",
        "started_at",
        "finished_at",
        "finished_latitude",
        "finished_longitude",
        "created_at",
        "updated_at",
        "car_id",
        "car_name",
        "car_police_number",
        "car_tracker_id",
        "tracker_id",
        "tracker_name",
        "contact_id",
        "contact_name",
        "contact_latitude",
        "contact_longitude",
    ])
    .map_err(|_| Status::InternalServerError)?;

    for t in raw_trips {
        let car = t.car_id.and_then(|cid| related_cars.iter().find(|c| c.car_id == cid));
        let contact = related_contacts.iter().find(|c| c.contact_id == t.contact_id);
        let tracker = t.tracker_id.and_then(|tid| related_trackers.iter().find(|tr| tr.tracker_id == tid));

        wtr.write_record(&[
            t.activity_id.to_string(),
            format!("{:?}", t.activity_type),
            t.description.clone().unwrap_or_default(),
            t.started_at.map(|dt| dt.to_string()).unwrap_or_default(),
            t.finished_at.map(|dt| dt.to_string()).unwrap_or_default(),
            t.finished_latitude.map(|val| val.to_string()).unwrap_or_default(),
            t.finished_longitude.map(|val| val.to_string()).unwrap_or_default(),
            t.created_at.to_string(),
            t.updated_at.to_string(),
            t.car_id.map(|id| id.to_string()).unwrap_or_default(),
            car.map(|c| c.name.clone()).unwrap_or_default(),
            car.map(|c| c.police_number.clone()).unwrap_or_default(),
            car.and_then(|c| c.tracker_id).map(|id| id.to_string()).unwrap_or_default(),
            t.tracker_id.map(|id| id.to_string()).unwrap_or_default(),
            tracker.map(|tr| tr.name.clone()).unwrap_or_default(),
            t.contact_id.to_string(),
            contact.map(|c| c.name.clone()).unwrap_or_default(),
            contact.map(|c| c.latitude.to_string()).unwrap_or_default(),
            contact.map(|c| c.longitude.to_string()).unwrap_or_default(),
        ])
        .map_err(|_| Status::InternalServerError)?;
    }

    let csv_bytes = wtr.into_inner().map_err(|_| Status::InternalServerError)?;

    Ok(CsvResponse(csv_bytes))
}
