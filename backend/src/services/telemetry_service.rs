use crate::{error::ServiceResult, services::metrics};
use actix_web::http::StatusCode;
use bson::doc;
use chrono::Utc;
use mongodb::Database;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::error::ServiceError;

const COLLECTION: &str = "page_views";
const RATE_LIMIT: usize = 120;
const RATE_WINDOW: Duration = Duration::from_secs(10 * 60);
const WINDOW_DAYS: i64 = 13;

#[derive(Debug, Deserialize)]
struct ViewRow {
    day: String,
    path: String,
    n: i64,
}

#[derive(Debug, Serialize)]
pub struct DayCount {
    pub day: String,
    pub n: i64,
}

#[derive(Debug, Serialize)]
pub struct PathCount {
    pub path: String,
    pub n: i64,
}

#[derive(Debug, Serialize)]
pub struct Analytics {
    pub days: Vec<DayCount>,
    pub paths: Vec<PathCount>,
}

fn allow(ip: &str) -> bool {
    static HITS: Mutex<Option<HashMap<String, Vec<Instant>>>> = Mutex::new(None);
    let mut guard = HITS.lock().unwrap_or_else(|err| err.into_inner());
    let hits = guard.get_or_insert_with(HashMap::new);
    let now = Instant::now();
    let recent = hits.entry(ip.to_string()).or_default();
    recent.retain(|at| now.duration_since(*at) < RATE_WINDOW);
    if recent.len() >= RATE_LIMIT {
        return false;
    }
    recent.push(now);
    true
}

pub fn clean_path(raw: &str) -> Result<String, &'static str> {
    let path = raw.split(['?', '#']).next().unwrap_or("").trim();
    if !(1..=80).contains(&path.len())
        || !path.starts_with('/')
        || path.contains("..")
        || path.contains("//")
    {
        return Err("BAD_PATH");
    }
    if !path
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '_' | '.'))
    {
        return Err("BAD_PATH");
    }
    Ok(path.to_string())
}

pub async fn ingest(raw_path: &str, client_ip: &str, db: &Database) -> ServiceResult<()> {
    let path = clean_path(raw_path).map_err(|err| ServiceError::new(StatusCode::BAD_REQUEST, err))?;
    if !allow(client_ip) {
        return Err(ServiceError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many events. Try again later.",
        ));
    }
    let day = Utc::now().format("%Y-%m-%d").to_string();
    db.collection::<bson::Document>(COLLECTION)
        .update_one(
            doc! { "day": &day, "path": &path },
            doc! { "$inc": { "n": 1_i64 } },
        )
        .upsert(true)
        .await
        .map_err(|err| {
            log::warn!("page view save failed: {err}");
            ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, "SAVE_FAILED")
        })?;
    metrics::record_telemetry();
    Ok(())
}

pub async fn summary(db: &Database) -> Analytics {
    let since = (Utc::now().date_naive() - chrono::Duration::days(WINDOW_DAYS))
        .format("%Y-%m-%d")
        .to_string();
    let coll = db.collection::<ViewRow>(COLLECTION);
    let mut rows = Vec::new();
    if let Ok(mut cursor) = coll.find(doc! { "day": { "$gte": &since } }).await {
        while cursor.advance().await.unwrap_or(false) {
            if let Ok(row) = cursor.deserialize_current() {
                rows.push(row);
            }
        }
    }

    let mut days: HashMap<String, i64> = HashMap::new();
    let mut paths: HashMap<String, i64> = HashMap::new();
    for row in rows {
        *days.entry(row.day).or_insert(0) += row.n;
        *paths.entry(row.path).or_insert(0) += row.n;
    }
    let mut days: Vec<DayCount> = days.into_iter().map(|(day, n)| DayCount { day, n }).collect();
    days.sort_by(|a, b| a.day.cmp(&b.day));
    let mut paths: Vec<PathCount> = paths
        .into_iter()
        .map(|(path, n)| PathCount { path, n })
        .collect();
    paths.sort_by(|a, b| b.n.cmp(&a.n).then(a.path.cmp(&b.path)));
    paths.truncate(20);
    Analytics { days, paths }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_path_keeps_a_page_and_drops_the_rest() {
        assert_eq!(clean_path("/galleries/laguna-seca"), Ok("/galleries/laguna-seca".into()));
        assert_eq!(clean_path("/search?q=grid"), Ok("/search".into()));
        assert!(clean_path("https://evil.example").is_err());
        assert!(clean_path("/../secret").is_err());
        assert!(clean_path("/a b").is_err());
    }

    #[test]
    fn rate_limit_is_120_in_the_window() {
        let ip = format!("telemetry-test-{}", std::process::id());
        for _ in 0..120 {
            assert!(allow(&ip));
        }
        assert!(!allow(&ip));
    }
}
