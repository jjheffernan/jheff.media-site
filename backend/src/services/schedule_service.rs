use crate::{
    error::{ServiceError, ServiceResult},
    models::schedule::{CreateScheduleEvent, ScheduleConfig, ScheduleEvent, ScheduleEventRecord},
};
use actix_web::http::StatusCode;
use bson::{doc, oid::ObjectId};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use mongodb::Database;
use std::env;

const SCHEDULE_EVENTS: &str = "schedule_events";

pub async fn schedule_config(db: &Database) -> ScheduleConfig {
    let embed_url = env::var("YEW_FULLSTACK_SCHEDULE_EMBED_URL")
        .ok()
        .filter(|url| !url.is_empty());
    let source = if embed_url.is_some() {
        String::from("embed")
    } else {
        String::from("none")
    };

    let events = match list_records(db).await {
        Ok(records) => upcoming_events(records, Utc::now()),
        Err(err) => {
            error!("schedule events unavailable: {err}");
            vec![]
        }
    };

    ScheduleConfig {
        embed_url,
        source,
        events,
    }
}

pub async fn create_event(input: CreateScheduleEvent, db: &Database) -> ServiceResult<String> {
    let record = validate_event(input)?;
    let coll = db.collection::<ScheduleEventRecord>(SCHEDULE_EVENTS);
    let result = coll
        .insert_one(&record)
        .await
        .map_err(|err| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    result
        .inserted_id
        .as_object_id()
        .map(|id| id.to_hex())
        .ok_or_else(|| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, "CREATE_FAILED"))
}

pub async fn delete_event(id: &str, db: &Database) -> ServiceResult<()> {
    let oid = ObjectId::parse_str(id)
        .map_err(|_| ServiceError::new(StatusCode::BAD_REQUEST, "INVALID_ID"))?;
    let coll = db.collection::<ScheduleEventRecord>(SCHEDULE_EVENTS);
    let result = coll
        .delete_one(doc! { "_id": oid })
        .await
        .map_err(|err| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    if result.deleted_count == 0 {
        return Err(ServiceError::new(StatusCode::NOT_FOUND, "NOT_FOUND"));
    }
    Ok(())
}

fn validate_event(input: CreateScheduleEvent) -> ServiceResult<ScheduleEventRecord> {
    let title = input.title.trim().to_string();
    let starts_at = input.starts_at.trim().to_string();
    let kind = normalize_kind(input.kind.trim()).map(str::to_string);
    if title.is_empty() || parse_starts_at(&starts_at).is_none() || kind.is_none() {
        return Err(ServiceError::new(StatusCode::BAD_REQUEST, "INVALID_EVENT"));
    }
    let location = input
        .location
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    Ok(ScheduleEventRecord {
        id: None,
        title,
        starts_at,
        location,
        kind: kind.unwrap_or_default(),
        public: true,
    })
}

fn normalize_kind(kind: &str) -> Option<&'static str> {
    match kind {
        "shoot" => Some("shoot"),
        "instruction" => Some("instruction"),
        "other" => Some("other"),
        _ => None,
    }
}

fn parse_starts_at(value: &str) -> Option<DateTime<Utc>> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.with_timezone(&Utc));
    }
    for format in [
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
    ] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(value, format) {
            return Some(Utc.from_utc_datetime(&naive));
        }
    }
    None
}

async fn list_records(db: &Database) -> mongodb::error::Result<Vec<ScheduleEventRecord>> {
    let coll = db.collection::<ScheduleEventRecord>(SCHEDULE_EVENTS);
    let mut cursor = coll.find(doc! {}).await?;
    let mut records = vec![];
    while cursor.advance().await? {
        if let Ok(record) = cursor.deserialize_current() {
            records.push(record);
        }
    }
    Ok(records)
}

fn upcoming_events(records: Vec<ScheduleEventRecord>, now: DateTime<Utc>) -> Vec<ScheduleEvent> {
    let mut upcoming = Vec::new();
    for record in records {
        if !record.public {
            continue;
        }
        let Some(starts) = parse_starts_at(&record.starts_at) else {
            continue;
        };
        if starts <= now {
            continue;
        }
        let Some(id) = record.id.map(|oid| oid.to_hex()) else {
            continue;
        };
        let location = record.location.filter(|value| !value.is_empty());
        upcoming.push((
            starts,
            ScheduleEvent {
                id,
                title: record.title,
                starts_at: record.starts_at,
                location,
                kind: record.kind,
            },
        ));
    }
    upcoming.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.title.cmp(&b.1.title))
            .then_with(|| a.1.id.cmp(&b.1.id))
    });
    upcoming.into_iter().map(|(_, event)| event).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bson::oid::ObjectId;

    fn at(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn record(title: &str, starts_at: &str, public: bool) -> ScheduleEventRecord {
        ScheduleEventRecord {
            id: Some(ObjectId::new()),
            title: title.into(),
            starts_at: starts_at.into(),
            location: Some("PittRace".into()),
            kind: "shoot".into(),
            public,
        }
    }

    #[test]
    fn parses_datetime_local_and_rfc3339() {
        assert_eq!(
            parse_starts_at("2026-11-02T15:45"),
            Some(at("2026-11-02T15:45:00Z"))
        );
        assert_eq!(
            parse_starts_at("2026-11-02T15:45:30"),
            Some(at("2026-11-02T15:45:30Z"))
        );
        assert_eq!(
            parse_starts_at("2026-11-02T15:45:30.5"),
            Some(at("2026-11-02T15:45:30.500Z"))
        );
        assert_eq!(
            parse_starts_at("2026-11-02T15:45:30Z"),
            Some(at("2026-11-02T15:45:30Z"))
        );
        assert!(parse_starts_at("not-a-date").is_none());
        assert!(parse_starts_at("").is_none());
    }

    #[test]
    fn upcoming_events_are_public_future_and_oldest_first() {
        let now = at("2026-06-01T12:00:00Z");
        let mut past = record("past", "2026-05-01T12:00", true);
        past.kind = "other".into();
        let mut equal = record("now", "2026-06-01T12:00:00Z", true);
        equal.kind = "instruction".into();
        let mut later = record("july", "2026-07-01T09:00", true);
        later.kind = "shoot".into();
        let mut sooner = record("june", "2026-06-15T18:30", true);
        sooner.location = None;
        let private = record("hidden", "2026-08-01T00:00", false);
        let mut broken = record("broken", "next tuesday", true);
        broken.public = true;

        let events = upcoming_events(vec![later, past, private, sooner, equal, broken], now);
        let titles: Vec<_> = events.iter().map(|event| event.title.as_str()).collect();
        assert_eq!(titles, ["june", "july"]);
        assert!(events[0].location.is_none());
        assert_eq!(events[1].location.as_deref(), Some("PittRace"));
        assert_eq!(events[1].kind, "shoot");
        assert_eq!(events[0].starts_at, "2026-06-15T18:30");
    }

    #[test]
    fn missing_public_defaults_to_true() {
        let record: ScheduleEventRecord = serde_json::from_str(
            r#"{"title":"Track day","startsAt":"2099-01-01T10:00","kind":"shoot"}"#,
        )
        .unwrap();
        assert!(record.public);
        assert!(record.location.is_none());
    }

    #[test]
    fn validate_event_keeps_the_typed_start_and_rejects_bad_input() {
        let record = match validate_event(CreateScheduleEvent {
            title: "  HPDE  ".into(),
            starts_at: "2026-11-02T15:45".into(),
            location: Some("  ".into()),
            kind: "instruction".into(),
        }) {
            Ok(record) => record,
            Err(err) => panic!("{}", err.body.message),
        };
        assert_eq!(record.title, "HPDE");
        assert_eq!(record.starts_at, "2026-11-02T15:45");
        assert!(record.location.is_none());
        assert_eq!(record.kind, "instruction");
        assert!(record.public);
        assert!(record.id.is_none());

        let err = validate_event(CreateScheduleEvent {
            title: " ".into(),
            starts_at: "2026-11-02T15:45".into(),
            location: None,
            kind: "shoot".into(),
        })
        .unwrap_err();
        assert_eq!(err.http_status, StatusCode::BAD_REQUEST);
        assert_eq!(err.body.message, "INVALID_EVENT");

        let err = validate_event(CreateScheduleEvent {
            title: "Clinic".into(),
            starts_at: "tomorrow".into(),
            location: None,
            kind: "other".into(),
        })
        .unwrap_err();
        assert_eq!(err.body.message, "INVALID_EVENT");

        let err = validate_event(CreateScheduleEvent {
            title: "Clinic".into(),
            starts_at: "2026-11-02T15:45".into(),
            location: None,
            kind: "party".into(),
        })
        .unwrap_err();
        assert_eq!(err.body.message, "INVALID_EVENT");
    }

    #[test]
    fn schedule_config_shape_keeps_embed_fields() {
        let config = ScheduleConfig {
            embed_url: Some("https://cal.example".into()),
            source: "embed".into(),
            events: vec![ScheduleEvent {
                id: "abc".into(),
                title: "HPDE".into(),
                starts_at: "2026-11-02T15:45".into(),
                location: None,
                kind: "instruction".into(),
            }],
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["embedUrl"], "https://cal.example");
        assert_eq!(json["source"], "embed");
        assert_eq!(json["events"][0]["startsAt"], "2026-11-02T15:45");
        assert_eq!(json["events"][0]["kind"], "instruction");
        assert!(json["events"][0].get("location").is_none());
        assert!(json["events"][0].get("public").is_none());
    }
}
