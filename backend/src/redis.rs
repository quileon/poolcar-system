use crate::{error::MqttError, models::activity::ActivityDetails};
use deadpool_redis::redis::AsyncCommands;
use rust_decimal::Decimal;

pub async fn reload_redis_activities(
    db: &sqlx::MySqlPool,
    redis: &deadpool_redis::Pool,
) -> Result<(), MqttError> {
    let active_activities: Vec<ActivityDetails> = sqlx::query_as(
        r#"
            SELECT
                activities.activity_id,
                activities.car_id,
                cars.name AS car_name,
                cars.police_number AS car_police_number,
                activities.contact_id,
                contacts.name AS contact_name,
                contacts.latitude AS contact_latitude,
                contacts.longitude AS contact_longitude,
                activities.activity_type_id,
                activity_types.name AS activity_type_name,
                activities.tracker_id,
                trackers.name AS tracker_name,
                activities.started_at,
                activities.finished_at,
                activities.finished_latitude,
                activities.finished_longitude,
                activities.description,
                activities.created_at,
                activities.updated_at,
                activities.deleted_at
            FROM activities
            LEFT JOIN cars ON cars.car_id = activities.car_id
            LEFT JOIN contacts ON contacts.contact_id = activities.contact_id
            LEFT JOIN activity_types ON activity_types.activity_type_id = activities.activity_type_id
            LEFT JOIN trackers ON trackers.tracker_id = activities.tracker_id
            WHERE activities.deleted_at IS NULL
                AND activities.finished_at IS NULL
            ORDER BY activities.activity_id ASC
        "#
    ).fetch_all(db).await?;

    let active_activities = serde_json::to_string(&active_activities)?;
    let mut conn = redis.get().await?;

    conn.set::<_, _, ()>("activities", active_activities)
        .await?;

    Ok(())
}

pub async fn get_all_redis_activities(
    redis: &deadpool_redis::Pool,
) -> Result<Vec<ActivityDetails>, MqttError> {
    let mut conn = redis.get().await?;
    let active_activities: String = conn.get("activities").await?;
    let active_activities: Vec<ActivityDetails> = serde_json::from_str(&active_activities)?;

    Ok(active_activities)
}

pub async fn complete_redis_activities(
    db: &sqlx::MySqlPool,
    redis: &deadpool_redis::Pool,
    activity_id: i32,
    tracker_id: u8,
    finished_latitude: Decimal,
    finished_longitude: Decimal,
) -> Result<(), MqttError> {
    let car_id: Option<i32> = sqlx::query_scalar(
        r#"
            SELECT car_id FROM cars
            WHERE tracker_id = ? AND deleted_at IS NULL
        "#,
    )
    .bind(tracker_id as i32)
    .fetch_optional(db)
    .await?
    .flatten();

    sqlx::query(
        r#"
            UPDATE activities
            SET finished_at = CURRENT_TIMESTAMP, finished_latitude = ?, finished_longitude = ?, tracker_id = ?, car_id = ?
            WHERE activity_id = ?
        "#
    )
    .bind(finished_latitude)
    .bind(finished_longitude)
    .bind(tracker_id as i32)
    .bind(car_id)
    .bind(activity_id)
    .execute(db)
    .await?;

    reload_redis_activities(db, redis).await?;

    Ok(())
}
