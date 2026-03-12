use crate::{
    error::MqttError,
    models::activity::{Activity, ActivityDetails},
};
use deadpool_redis::redis::AsyncCommands;
use rust_decimal::Decimal;

pub async fn reload_redis_activities(
    db: &sqlx::PgPool,
    redis: &deadpool_redis::Pool,
) -> Result<(), MqttError> {
    let active_activities = sqlx::query_as!(
        ActivityDetails,
        r#"
            SELECT
                activities.activity_id,
                activities.car_id,
                cars.name AS "car_name?",
                cars.police_number AS "car_police_number?",
                activities.contact_id,
                contacts.name AS contact_name,
                contacts.latitude AS contact_latitude,
                contacts.longitude AS contact_longitude,
                activities.activity_type_id,
                activity_types.name AS activity_type_name,
                activities.tracker_id,
                trackers.name AS "tracker_name?",
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
    db: &sqlx::PgPool,
    redis: &deadpool_redis::Pool,
    activity_id: i32,
    tracker_id: u8,
    finished_latitude: Decimal,
    finished_longitude: Decimal,
) -> Result<(), MqttError> {
    let car = sqlx::query!(
        r#"
            SELECT car_id FROM cars
            WHERE tracker_id = $1 AND deleted_at IS NULL
        "#,
        tracker_id as i32
    )
    .fetch_optional(db)
    .await?;

    let car_id = car.map(|c| c.car_id);

    sqlx::query_as!(
        Activity,
        r#"
            UPDATE activities
            SET finished_at = NOW(), finished_latitude = $2, finished_longitude = $3, tracker_id = $4, car_id = $5
            WHERE activity_id = $1
            RETURNING *
        "#,
        activity_id,
        finished_latitude,
        finished_longitude,
        tracker_id as i32,
        car_id
    )
    .fetch_one(db)
    .await?;

    reload_redis_activities(db, redis).await?;

    Ok(())
}
