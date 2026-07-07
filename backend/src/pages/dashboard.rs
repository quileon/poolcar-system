use crate::auth::AuthenticatedUser;
use crate::entities::sea_orm_active_enums::ActivityType;
use crate::entities::{activities, car_status, cars, contacts, trackers};
use askama::Template;
use askama_web::WebTemplate;
use rocket::State;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect,
};

#[derive(Clone)]
pub struct FormattedActivity {
    pub is_active: bool,
    pub time_ago: String,
    pub contact_name: String,
    pub activity_type: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "dashboard.j2")]
pub struct DashboardTemplate {
    pub error: Option<String>,
    pub username: String,
    pub role: String,
    pub active_activities_today: u64,
    pub all_activities_today: u64,
    pub assigned_cars: u64,
    pub all_cars: u64,
    pub total_trackers: u64,
    pub car_checks_today: u64,
    pub activities: Vec<FormattedActivity>,
}

#[rocket::get("/")]
pub async fn dashboard(
    user: AuthenticatedUser,
    db: &State<DatabaseConnection>,
) -> DashboardTemplate {
    let wib_now = chrono::Utc::now() + chrono::Duration::hours(7);
    let wib_today = wib_now.date_naive();
    let wib_start_local =
        chrono::NaiveDateTime::new(wib_today, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    let wib_end_local = chrono::NaiveDateTime::new(
        wib_today,
        chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
    );

    let start_utc = wib_start_local - chrono::Duration::hours(7);
    let end_utc = wib_end_local - chrono::Duration::hours(7);

    // Fetch counts
    let active_activities_today = activities::Entity::find()
        .filter(activities::Column::DeletedAt.is_null())
        .filter(activities::Column::StartedAt.gte(start_utc))
        .filter(activities::Column::StartedAt.lte(end_utc))
        .filter(activities::Column::FinishedAt.is_null())
        .count(db.inner())
        .await
        .unwrap_or(0);

    let all_activities_today = activities::Entity::find()
        .filter(activities::Column::DeletedAt.is_null())
        .filter(activities::Column::StartedAt.gte(start_utc))
        .filter(activities::Column::StartedAt.lte(end_utc))
        .count(db.inner())
        .await
        .unwrap_or(0);

    let assigned_cars = cars::Entity::find()
        .filter(cars::Column::DeletedAt.is_null())
        .filter(cars::Column::TrackerId.is_not_null())
        .count(db.inner())
        .await
        .unwrap_or(0);

    let all_cars = cars::Entity::find()
        .filter(cars::Column::DeletedAt.is_null())
        .count(db.inner())
        .await
        .unwrap_or(0);

    let total_trackers = trackers::Entity::find()
        .filter(trackers::Column::DeletedAt.is_null())
        .count(db.inner())
        .await
        .unwrap_or(0);

    let car_checks_today = car_status::Entity::find()
        .filter(car_status::Column::DeletedAt.is_null())
        .filter(car_status::Column::RecordedAt.gte(start_utc))
        .filter(car_status::Column::RecordedAt.lte(end_utc))
        .count(db.inner())
        .await
        .unwrap_or(0);

    let one_week_ago = chrono::Utc::now().naive_utc() - chrono::Duration::days(7);

    // Query recent activities from the last 7 days with contacts
    let recent_activities_query = activities::Entity::find()
        .filter(activities::Column::DeletedAt.is_null())
        .filter(activities::Column::CreatedAt.gte(one_week_ago))
        .find_also_related(contacts::Entity)
        .order_by_desc(activities::Column::CreatedAt)
        .all(db.inner())
        .await;

    let mut formatted_activities = Vec::new();
    let mut error = None;

    match recent_activities_query {
        Ok(items) => {
            for (activity, contact_opt) in items {
                let is_active = activity.finished_at.is_none();

                // Calculate time ago using started_at or created_at (fallback to created_at)
                let time_ago = format_relative_time(activity.created_at);

                let contact_name = if let Some(contact) = contact_opt {
                    contact.name
                } else {
                    "Unknown Contact".to_string()
                };

                let activity_type = match activity.activity_type {
                    ActivityType::Delivery => "Delivery".to_string(),
                    ActivityType::Meeting => "Meeting".to_string(),
                    ActivityType::TrialT1 => "Trial T1".to_string(),
                };

                formatted_activities.push(FormattedActivity {
                    is_active,
                    time_ago,
                    contact_name,
                    activity_type,
                });
            }
        }
        Err(err) => {
            error = Some(err.to_string());
        }
    }

    DashboardTemplate {
        error,
        username: user.username,
        role: user.role,
        active_activities_today,
        all_activities_today,
        assigned_cars,
        all_cars,
        total_trackers,
        car_checks_today,
        activities: formatted_activities,
    }
}

fn format_relative_time(dt: chrono::NaiveDateTime) -> String {
    let now = chrono::Utc::now().naive_utc();
    let duration = now.signed_duration_since(dt);

    if duration.num_seconds() < 0 {
        return "Just now".to_string();
    }

    let mins = duration.num_minutes();
    if mins < 1 {
        return "Just now".to_string();
    }
    if mins < 60 {
        if mins == 1 {
            return "1 min ago".to_string();
        }
        return format!("{} mins ago", mins);
    }

    let hours = duration.num_hours();
    if hours < 24 {
        if hours == 1 {
            return "1 hour ago".to_string();
        }
        return format!("{} hours ago", hours);
    }

    let days = duration.num_days();
    if days == 1 {
        return "1 day ago".to_string();
    }
    format!("{} days ago", days)
}
