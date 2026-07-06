use crate::auth::AuthenticatedUser;
use crate::entities::{audit, cars, trackers};
use askama::Template;
use askama_web::WebTemplate;
use rocket::State;
use rocket::http::Status;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

pub struct CsvResponse(pub Vec<u8>);

impl<'r> rocket::response::Responder<'r, 'static> for CsvResponse {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        use rocket::http::{ContentType, Header};
        rocket::response::Response::build()
            .header(ContentType::CSV)
            .header(Header::new(
                "Content-Disposition",
                "attachment; filename=\"audit_export.csv\"",
            ))
            .sized_body(self.0.len(), std::io::Cursor::new(self.0))
            .ok()
    }
}

pub struct FormattedAuditRecord {
    pub audit_id: i64,
    pub car_id: Option<i32>,
    pub tracker_id: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub recorded_at_wib: String,
    pub time_wib: String,
    pub created_at_wib: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "audit.j2")]
pub struct AuditTemplate {
    pub username: String,
    pub role: String,
    pub trackers: Vec<trackers::Model>,
    pub cars: Vec<cars::Model>,
    pub selected_choice: Option<String>,
    pub selected_target: Option<i32>,
    pub selected_date: Option<String>,
    pub audit_records: Vec<FormattedAuditRecord>,
    pub error: Option<String>,
}

#[rocket::get("/audit?<choice>&<target>&<date>")]
pub async fn audit_page(
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
    choice: Option<String>,
    target: Option<i32>,
    date: Option<String>,
) -> Result<AuditTemplate, Status> {
    if user.role == "Security" {
        return Err(Status::Forbidden);
    }

    let trackers = match trackers::Entity::find().all(db.inner()).await {
        Ok(t) => t,
        Err(err) => {
            return Ok(AuditTemplate {
                username: user.username.clone(),
                role: user.role.clone(),
                trackers: vec![],
                cars: vec![],
                selected_choice: None,
                selected_target: None,
                selected_date: None,
                audit_records: vec![],
                error: Some(err.to_string()),
            });
        }
    };

    let cars = match cars::Entity::find().all(db.inner()).await {
        Ok(c) => c,
        Err(err) => {
            return Ok(AuditTemplate {
                username: user.username.clone(),
                role: user.role.clone(),
                trackers,
                cars: vec![],
                selected_choice: None,
                selected_target: None,
                selected_date: None,
                audit_records: vec![],
                error: Some(err.to_string()),
            });
        }
    };

    let mut audit_records = vec![];
    if let (Some(choice), Some(target), Some(date)) = (choice.as_ref(), target, date.as_ref()) {
        let mut query = audit::Entity::find();
        if choice == "tracker" {
            query = query.filter(audit::Column::TrackerId.eq(target));
        } else if choice == "car" {
            query = query.filter(audit::Column::CarId.eq(target));
        }
        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            let local_start = naive_date.and_hms_opt(0, 0, 0).unwrap();
            let local_end = naive_date.and_hms_opt(23, 59, 59).unwrap();

            // WIB is UTC+7, so convert local WIB times to UTC by subtracting 7 hours
            let start_date = local_start - chrono::Duration::hours(7);
            let end_date = local_end - chrono::Duration::hours(7);
            query = query.filter(audit::Column::RecordedAt.between(start_date, end_date));
        }
        query = query.order_by_asc(audit::Column::RecordedAt);

        match query.all(db.inner()).await {
            Ok(records) => {
                audit_records = records
                    .into_iter()
                    .map(|record| {
                        let recorded_wib = record.recorded_at + chrono::Duration::hours(7);
                        let created_wib = record.created_at + chrono::Duration::hours(7);
                        FormattedAuditRecord {
                            audit_id: record.audit_id,
                            car_id: record.car_id,
                            tracker_id: record.tracker_id,
                            latitude: record.latitude,
                            longitude: record.longitude,
                            recorded_at_wib: recorded_wib.format("%Y-%m-%d %H:%M:%S").to_string(),
                            time_wib: recorded_wib.format("%H:%M:%S").to_string(),
                            created_at_wib: created_wib.format("%Y-%m-%d %H:%M:%S").to_string(),
                        }
                    })
                    .collect();
            }
            Err(err) => {
                return Ok(AuditTemplate {
                    username: user.username,
                    role: user.role,
                    trackers,
                    cars,
                    selected_choice: Some(choice.clone()),
                    selected_target: Some(target),
                    selected_date: Some(date.clone()),
                    audit_records: vec![],
                    error: Some(err.to_string()),
                });
            }
        }
    }

    Ok(AuditTemplate {
        username: user.username,
        role: user.role,
        trackers,
        cars,
        selected_choice: choice,
        selected_target: target,
        selected_date: date,
        audit_records,
        error: None,
    })
}

#[rocket::get("/audit/export?<choice>&<target>&<date>")]
pub async fn export_audit_csv(
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
    choice: Option<String>,
    target: Option<i32>,
    date: Option<String>,
) -> Result<CsvResponse, Status> {
    if user.role == "Security" {
        return Err(Status::Forbidden);
    }

    let mut query = audit::Entity::find();
    if let (Some(choice), Some(target)) = (choice.as_ref(), target) {
        if choice == "tracker" {
            query = query.filter(audit::Column::TrackerId.eq(target));
        } else if choice == "car" {
            query = query.filter(audit::Column::CarId.eq(target));
        }
    }
    if let Some(ref date) = date {
        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            let local_start = naive_date.and_hms_opt(0, 0, 0).unwrap();
            let local_end = naive_date.and_hms_opt(23, 59, 59).unwrap();

            // WIB is UTC+7, so convert local WIB times to UTC by subtracting 7 hours
            let start_date = local_start - chrono::Duration::hours(7);
            let end_date = local_end - chrono::Duration::hours(7);
            query = query.filter(audit::Column::RecordedAt.between(start_date, end_date));
        }
    }
    query = query.order_by_asc(audit::Column::RecordedAt);

    let audit_records = query
        .all(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    let mut writer = csv::Writer::from_writer(Vec::new());
    writer
        .write_record(&[
            "Audit ID",
            "Car ID",
            "Tracker ID",
            "Latitude",
            "Longitude",
            "Recorded At (WIB)",
            "Created At (WIB)",
        ])
        .map_err(|_| Status::InternalServerError)?;
    for record in audit_records {
        let recorded_wib = record.recorded_at + chrono::Duration::hours(7);
        let created_wib = record.created_at + chrono::Duration::hours(7);
        writer
            .write_record(&[
                record.audit_id.to_string(),
                record.car_id.map(|id| id.to_string()).unwrap_or_default(),
                record.tracker_id.to_string(),
                record.latitude.to_string(),
                record.longitude.to_string(),
                recorded_wib.format("%Y-%m-%d %H:%M:%S").to_string(),
                created_wib.format("%Y-%m-%d %H:%M:%S").to_string(),
            ])
            .map_err(|_| Status::InternalServerError)?;
    }
    let data = writer
        .into_inner()
        .map_err(|_| Status::InternalServerError)?;
    Ok(CsvResponse(data))
}
