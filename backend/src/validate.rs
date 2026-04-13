use chrono::{Local, NaiveDate};

pub fn is_valid_date(date_str: &str) -> bool {
    if date_str.len() != 10 {
        return false;
    }

    let parts: Vec<&str> = date_str.split("-").collect();
    if parts.len() != 3 {
        return false;
    }

    let year: Result<u32, _> = parts[0].parse();
    let month: Result<u32, _> = parts[1].parse();
    let day: Result<u32, _> = parts[2].parse();

    match (year, month, day) {
        (Ok(y), Ok(m), Ok(d)) => y > 0 && m >= 1 && m <= 12 && d >= 1 && d <= 31,
        _ => false,
    }
}

pub fn is_within_30_days(date_str: &str) -> bool {
    match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(target_date) => {
            let today = Local::now().naive_local().date();
            let duration = today.signed_duration_since(target_date);

            duration.num_days() >= 0 && duration.num_days() <= 30
        }
        _ => false,
    }
}
