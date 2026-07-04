use crate::auth::AuthenticatedUser;
use crate::entities::{activities as trips_entity, cars, contacts, trackers};
use askama::Template;
use askama_web::WebTemplate;
use chrono::NaiveDateTime;
use rocket::form::Form;
use rocket::http::Status;
use rocket::{FromForm, State};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use tracing::error;

pub struct TripWithDetails {
    pub trip: trips_entity::Model,
    pub car: Option<cars::Model>,
    pub contact: Option<contacts::Model>,
    pub tracker: Option<trackers::Model>,
}

#[derive(Template, WebTemplate)]
#[template(path = "trips.j2")]
pub struct TripsTemplate {
    pub username: String,
    pub role: String,
    pub trips: Vec<TripWithDetails>,
    pub cars: Vec<cars::Model>,
    pub contacts: Vec<contacts::Model>,
    pub trackers: Vec<trackers::Model>,
    pub editing_trip: Option<trips_entity::Model>,
    pub current_page: u64,
    pub total_pages: u64,
    pub pages: Vec<u64>,
    pub error: Option<String>,
}

#[derive(FromForm)]
pub struct TripForm<'r> {
    pub car_id: Option<i32>,
    pub contact_id: i32,
    pub activity_type: &'r str,
    pub tracker_id: Option<i32>,
    pub started_at: Option<&'r str>,
    pub finished_at: Option<&'r str>,
    pub finished_latitude: Option<f64>,
    pub finished_longitude: Option<f64>,
    pub description: Option<&'r str>,
}

pub fn parse_datetime(s: Option<&str>) -> Option<NaiveDateTime> {
    s.and_then(|val| {
        if val.trim().is_empty() {
            None
        } else {
            NaiveDateTime::parse_from_str(val, "%Y-%m-%dT%H:%M").ok()
        }
    })
}

#[derive(Serialize, Deserialize)]
pub struct ActiveTripCache {
    pub trip: trips_entity::Model,
    pub contact_latitude: f64,
    pub contact_longitude: f64,
}

pub async fn reload_active_trips_cache(
    db: &DatabaseConnection,
    redis: &redis::Client,
) -> Result<(), anyhow::Error> {
    let active_trips_with_contacts = trips_entity::Entity::find()
        .filter(trips_entity::Column::FinishedAt.is_null())
        .find_also_related(contacts::Entity)
        .all(db)
        .await?;

    let mut cache_items = Vec::new();
    for (trip, contact_opt) in active_trips_with_contacts {
        if let Some(contact) = contact_opt {
            cache_items.push(ActiveTripCache {
                trip,
                contact_latitude: contact.latitude,
                contact_longitude: contact.longitude,
            });
        }
    }

    let json_data = serde_json::to_string(&cache_items)?;

    use redis::AsyncCommands;
    let mut redis_conn = redis.get_multiplexed_async_connection().await?;
    redis_conn
        .set::<_, _, ()>("trips:active", &json_data)
        .await?;

    Ok(())
}

pub async fn get_active_trips(
    db: &DatabaseConnection,
    redis: &redis::Client,
) -> Result<Vec<ActiveTripCache>, anyhow::Error> {
    use redis::AsyncCommands;
    let mut redis_conn = redis.get_multiplexed_async_connection().await?;

    if let Ok(Some(cached_json)) = redis_conn.get::<_, Option<String>>("trips:active").await {
        if let Ok(trips) = serde_json::from_str::<Vec<ActiveTripCache>>(&cached_json) {
            return Ok(trips);
        }
    }

    let active_trips_with_contacts = trips_entity::Entity::find()
        .filter(trips_entity::Column::FinishedAt.is_null())
        .find_also_related(contacts::Entity)
        .all(db)
        .await?;

    let mut cache_items = Vec::new();
    for (trip, contact_opt) in active_trips_with_contacts {
        if let Some(contact) = contact_opt {
            cache_items.push(ActiveTripCache {
                trip,
                contact_latitude: contact.latitude,
                contact_longitude: contact.longitude,
            });
        }
    }

    let json_data = serde_json::to_string(&cache_items)?;
    let _ = redis_conn.set::<_, _, ()>("trips:active", &json_data).await;

    Ok(cache_items)
}

pub async fn finish_trip(
    activity_id: i32,
    tracker_id: i32,
    finished_latitude: f64,
    finished_longitude: f64,
    db: &DatabaseConnection,
    redis: &redis::Client,
    tx: &tokio::sync::broadcast::Sender<String>,
) -> Result<(), anyhow::Error> {
    let trip = trips_entity::Entity::find_by_id(activity_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Trip with ID {} not found", activity_id))?;

    let assigned_car = cars::Entity::find()
        .filter(cars::Column::TrackerId.eq(tracker_id))
        .filter(cars::Column::DeletedAt.is_null())
        .one(db)
        .await?;
    let car_id = assigned_car.map(|c| c.car_id);

    let now = chrono::Utc::now().naive_utc();

    let mut active: trips_entity::ActiveModel = trip.into();
    active.car_id = Set(car_id);
    active.tracker_id = Set(Some(tracker_id));
    active.finished_at = Set(Some(now));
    active.finished_latitude = Set(Some(finished_latitude));
    active.finished_longitude = Set(Some(finished_longitude));
    active.updated_at = Set(now);

    active.update(db).await?;

    if let Err(e) = reload_active_trips_cache(db, redis).await {
        error!("Failed to reload active trips cache: {:?}", e);
    }

    let ws_message = serde_json::json!({
        "message_type": "remove_destination",
        "data": {
            "activity_id": activity_id,
        }
    });
    let _ = tx.send(ws_message.to_string());

    Ok(())
}

async fn render_trips(
    db: &DatabaseConnection,
    user: &AuthenticatedUser,
    edit: Option<i32>,
    page: Option<u64>,
    error: Option<String>,
) -> Result<TripsTemplate, Status> {
    let current_page = page.unwrap_or(1);
    let page_size = 5;

    let editing_trip = match edit {
        Some(id) => trips_entity::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|_| Status::InternalServerError)?,
        None => None,
    };

    let all_cars = cars::Entity::find()
        .order_by_asc(cars::Column::Name)
        .all(db)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let all_contacts = contacts::Entity::find()
        .order_by_asc(contacts::Column::Name)
        .all(db)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let all_trackers = trackers::Entity::find()
        .order_by_asc(trackers::Column::Name)
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

    Ok(TripsTemplate {
        username: user.username.clone(),
        role: user.role.clone(),
        trips,
        cars: all_cars,
        contacts: all_contacts,
        trackers: all_trackers,
        editing_trip,
        current_page: target_page,
        total_pages,
        pages,
        error,
    })
}

#[rocket::get("/crud/trips?<edit>&<page>")]
pub async fn list_trips(
    edit: Option<i32>,
    page: Option<u64>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<TripsTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }
    render_trips(db.inner(), &user, edit, page, None).await
}

#[rocket::post("/crud/trips", data = "<form_data>")]
pub async fn create_trip(
    form_data: Form<TripForm<'_>>,
    db: &State<DatabaseConnection>,
    redis: &State<redis::Client>,
    tx: &State<tokio::sync::broadcast::Sender<String>>,
    user: AuthenticatedUser,
) -> Result<TripsTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let started_at = parse_datetime(form_data.started_at);
    let finished_at = parse_datetime(form_data.finished_at);

    let activity_type = match form_data.activity_type {
        "Delivery" => crate::entities::sea_orm_active_enums::ActivityType::Delivery,
        "Meeting" => crate::entities::sea_orm_active_enums::ActivityType::Meeting,
        _ => crate::entities::sea_orm_active_enums::ActivityType::TrialT1,
    };

    let now = chrono::Utc::now().naive_utc();

    let new_trip = trips_entity::ActiveModel {
        car_id: Set(form_data.car_id),
        contact_id: Set(form_data.contact_id),
        activity_type: Set(activity_type),
        tracker_id: Set(form_data.tracker_id),
        started_at: Set(started_at),
        finished_at: Set(finished_at),
        finished_latitude: Set(form_data.finished_latitude),
        finished_longitude: Set(form_data.finished_longitude),
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
                if let Ok(Some(contact)) = contacts::Entity::find_by_id(inserted.contact_id)
                    .one(db.inner())
                    .await
                {
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
            render_trips(db.inner(), &user, None, None, None).await
        }
        Err(err) => render_trips(db.inner(), &user, None, None, Some(err.to_string())).await,
    }
}

#[rocket::put("/crud/trips/<id>?<page>", data = "<form_data>")]
pub async fn update_trip(
    id: i32,
    page: Option<u64>,
    form_data: Form<TripForm<'_>>,
    db: &State<DatabaseConnection>,
    redis: &State<redis::Client>,
    tx: &State<tokio::sync::broadcast::Sender<String>>,
    user: AuthenticatedUser,
) -> Result<TripsTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let trip_record = match trips_entity::Entity::find_by_id(id).one(db.inner()).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            return render_trips(
                db.inner(),
                &user,
                Some(id),
                page,
                Some(format!("Trip with ID {} not found.", id)),
            )
            .await;
        }
        Err(err) => {
            return render_trips(db.inner(), &user, Some(id), page, Some(err.to_string())).await;
        }
    };

    let started_at = parse_datetime(form_data.started_at);
    let finished_at = parse_datetime(form_data.finished_at);

    let activity_type = match form_data.activity_type {
        "Delivery" => crate::entities::sea_orm_active_enums::ActivityType::Delivery,
        "Meeting" => crate::entities::sea_orm_active_enums::ActivityType::Meeting,
        _ => crate::entities::sea_orm_active_enums::ActivityType::TrialT1,
    };

    let mut active: trips_entity::ActiveModel = trip_record.into();
    active.car_id = Set(form_data.car_id);
    active.contact_id = Set(form_data.contact_id);
    active.activity_type = Set(activity_type);
    active.tracker_id = Set(form_data.tracker_id);
    active.started_at = Set(started_at);
    active.finished_at = Set(finished_at);
    active.finished_latitude = Set(form_data.finished_latitude);
    active.finished_longitude = Set(form_data.finished_longitude);
    active.description = Set(form_data.description.map(|s| s.to_string()));
    active.updated_at = Set(chrono::Utc::now().naive_utc());

    match active.update(db.inner()).await {
        Ok(_) => {
            if let Err(e) = reload_active_trips_cache(db.inner(), redis.inner()).await {
                error!("Failed to reload active trips cache: {:?}", e);
            }
            if finished_at.is_some() {
                let ws_message = serde_json::json!({
                    "message_type": "remove_destination",
                    "data": {
                        "activity_id": id,
                    }
                });
                let _ = tx.send(ws_message.to_string());
            } else {
                if let Ok(Some(contact)) = contacts::Entity::find_by_id(form_data.contact_id)
                    .one(db.inner())
                    .await
                {
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
            render_trips(db.inner(), &user, None, page, None).await
        }
        Err(err) => render_trips(db.inner(), &user, Some(id), page, Some(err.to_string())).await,
    }
}

#[rocket::delete("/crud/trips/<id>?<page>")]
pub async fn delete_trip(
    id: i32,
    page: Option<u64>,
    db: &State<DatabaseConnection>,
    redis: &State<redis::Client>,
    tx: &State<tokio::sync::broadcast::Sender<String>>,
    user: AuthenticatedUser,
) -> Result<TripsTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

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
            render_trips(db.inner(), &user, None, page, None).await
        }
        Err(err) => render_trips(db.inner(), &user, None, page, Some(err.to_string())).await,
    }
}
