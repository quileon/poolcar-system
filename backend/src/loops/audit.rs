use crate::{
    entities::{audit, cars, trackers},
    loops::mqtt::MqttPayload,
};
use haversine_rs::{point::Point, units::Unit};
use redis::AsyncCommands;
use sea_orm::{EntityTrait, Set};
use tracing::debug;

pub async fn audit_handler(
    db: &sea_orm::DatabaseConnection,
    redis: &redis::Client,
) -> anyhow::Result<()> {
    let tracker_cars = trackers::Entity::find()
        .find_also_related(cars::Entity)
        .all(db)
        .await?;

    let mut conn = redis.get_multiplexed_async_connection().await?;
    let mut audits_to_insert = Vec::new();
    let now = chrono::Utc::now().naive_utc();

    for (tracker, car) in tracker_cars {
        let key = format!("tracker:{}:live", tracker.tracker_id);
        let bytes_opt: Option<Vec<u8>> = conn.get(&key).await?;

        if let Some(bytes) = bytes_opt {
            if let Ok(payload) = serde_json::from_slice::<MqttPayload>(&bytes) {
                if let (Some(lat), Some(lng)) =
                    (payload.location.latitude, payload.location.longitude)
                {
                    // Compare if the car is moving from the last recorded position
                    let last_coordinates_str: Option<String> = conn
                        .get(format!("tracker:{}:last_coordinates", tracker.tracker_id))
                        .await?;

                    if let Some(ref json_str) = last_coordinates_str {
                        if let Ok(last_json) = serde_json::from_str::<serde_json::Value>(json_str) {
                            let last_lat = last_json["lat"].as_f64().unwrap_or(0.0);
                            let last_lng = last_json["lng"].as_f64().unwrap_or(0.0);
                            let p1 = Point::new(lat, lng);
                            let p2 = Point::new(last_lat, last_lng);
                            let distance = haversine_rs::distance(p1, p2, Unit::Meters);
                            if distance < 5.0 {
                                continue;
                            }
                        }
                    }

                    let last_coordinates_str =
                        serde_json::json!({ "lat": lat, "lng": lng }).to_string();
                    conn.set::<_, _, ()>(
                        format!("tracker:{}:last_coordinates", tracker.tracker_id),
                        last_coordinates_str,
                    )
                    .await?;

                    // Append for later saving into audit table
                    let recorded_at = if let Some(ref iso) = payload.datetime.iso8601 {
                        chrono::DateTime::parse_from_rfc3339(iso)
                            .map(|dt| dt.naive_utc())
                            .unwrap_or(now)
                    } else {
                        now
                    };

                    let audit_record = audit::ActiveModel {
                        car_id: Set(car.map(|c| c.car_id)),
                        tracker_id: Set(tracker.tracker_id),
                        latitude: Set(lat),
                        longitude: Set(lng),
                        recorded_at: Set(recorded_at),
                        created_at: Set(now),
                        updated_at: Set(now),
                        ..Default::default()
                    };
                    audits_to_insert.push(audit_record);
                }
            }
        }
    }

    debug!("Inserting {} audit records", audits_to_insert.len());
    if !audits_to_insert.is_empty() {
        audit::Entity::insert_many(audits_to_insert)
            .exec(db)
            .await?;
    }

    Ok(())
}
