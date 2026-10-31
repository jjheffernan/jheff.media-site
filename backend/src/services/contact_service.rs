use crate::{
    error::{ServiceError, ServiceResult},
    models::content::ContactSubmission,
};
use actix_web::http::StatusCode;
use bson::{doc, oid::ObjectId, DateTime};
use mongodb::Database;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const CONTACTS_COLLECTION: &str = "contacts";
const RATE_LIMIT: usize = 5;
const RATE_WINDOW: Duration = Duration::from_secs(10 * 60);

fn default_status() -> String {
    "new".into()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContactRecord {
    #[serde(rename = "_id", default, skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
    email: String,
    subject: String,
    message: String,
    kind: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    preferred_at: String,
    #[serde(default = "default_status")]
    status: String,
    created_at: DateTime,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactView {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
    pub preferred_at: String,
    pub created_at: String,
    pub status: String,
}

fn allow_submission(ip: &str) -> bool {
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

pub async fn submit_contact(
    form: ContactSubmission,
    client_ip: &str,
    db: &Database,
) -> ServiceResult<String> {
    if form.name.trim().is_empty() || form.email.trim().is_empty() || form.message.trim().is_empty()
    {
        return Err(ServiceError::new(
            StatusCode::BAD_REQUEST,
            "Name, email, and message are required.",
        ));
    }
    let kind = if form.kind.trim() == "booking" {
        "booking"
    } else {
        "contact"
    };
    let preferred_at = if kind == "booking" {
        clean_preferred(&form.preferred_at)?
    } else {
        String::new()
    };

    if !allow_submission(client_ip) {
        return Err(ServiceError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many messages. Try again later.",
        ));
    }

    if !form.website.trim().is_empty() {
        info!("Contact honeypot tripped from {client_ip}");
        return Ok(String::from(
            "Message received. We'll get back to you soon.",
        ));
    }

    let record = ContactRecord {
        id: None,
        name: form.name.trim().to_string(),
        email: form.email.trim().to_string(),
        subject: form.subject.trim().to_string(),
        message: form.message.trim().to_string(),
        kind: kind.to_string(),
        preferred_at,
        status: default_status(),
        created_at: DateTime::now(),
    };

    let collection = db.collection::<ContactRecord>(CONTACTS_COLLECTION);
    collection
        .insert_one(&record)
        .await
        .map_err(|err| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    info!(
        "Contact form received from {} <{}>",
        record.name, record.email
    );

    let mailed = if record.preferred_at.is_empty() {
        record.message.clone()
    } else {
        format!("{}\n\nPreferred: {}", record.message, record.preferred_at)
    };
    super::mailer::notify_contact(
        &record.name,
        &record.email,
        &record.subject,
        &mailed,
        &record.kind,
    )
    .await;

    Ok(String::from(
        "Message received. We'll get back to you soon.",
    ))
}

pub async fn list_contacts(db: &Database) -> ServiceResult<Vec<ContactView>> {
    let coll = db.collection::<ContactRecord>(CONTACTS_COLLECTION);
    let mut cursor = coll
        .find(doc! {})
        .sort(doc! { "createdAt": -1 })
        .await
        .map_err(|err| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let mut items = Vec::new();
    while cursor.advance().await.map_err(|err| {
        ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    })? {
        if let Ok(record) = cursor.deserialize_current() {
            if let Some(view) = contact_view(record) {
                items.push(view);
            }
        }
    }
    Ok(items)
}

pub async fn set_status(id: &str, status: &str, db: &Database) -> ServiceResult<()> {
    let oid = ObjectId::parse_str(id)
        .map_err(|_| ServiceError::new(StatusCode::BAD_REQUEST, "INVALID_ID"))?;
    let status = clean_status(status)?;
    let coll = db.collection::<ContactRecord>(CONTACTS_COLLECTION);
    let result = coll
        .update_one(doc! { "_id": oid }, doc! { "$set": { "status": &status } })
        .await
        .map_err(|err| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    if result.matched_count == 0 {
        return Err(ServiceError::new(StatusCode::NOT_FOUND, "NOT_FOUND"));
    }
    Ok(())
}

pub async fn delete_contact(id: &str, db: &Database) -> ServiceResult<()> {
    let oid = ObjectId::parse_str(id)
        .map_err(|_| ServiceError::new(StatusCode::BAD_REQUEST, "INVALID_ID"))?;
    let coll = db.collection::<ContactRecord>(CONTACTS_COLLECTION);
    let result = coll
        .delete_one(doc! { "_id": oid })
        .await
        .map_err(|err| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    if result.deleted_count == 0 {
        return Err(ServiceError::new(StatusCode::NOT_FOUND, "NOT_FOUND"));
    }
    Ok(())
}

fn contact_view(record: ContactRecord) -> Option<ContactView> {
    let id = record.id.map(|oid| oid.to_hex())?;
    let created_at = record
        .created_at
        .try_to_rfc3339_string()
        .unwrap_or_else(|_| record.created_at.timestamp_millis().to_string());
    Some(ContactView {
        id,
        kind: record.kind,
        name: record.name,
        email: record.email,
        subject: record.subject,
        message: record.message,
        preferred_at: record.preferred_at,
        created_at,
        status: record.status,
    })
}

fn clean_preferred(value: &str) -> ServiceResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(String::new());
    }
    if value.len() > 40 || super::schedule_service::parse_starts_at(value).is_none() {
        return Err(ServiceError::new(
            StatusCode::BAD_REQUEST,
            "Preferred time is not a date.",
        ));
    }
    Ok(value.to_string())
}

fn clean_status(status: &str) -> ServiceResult<String> {
    let status = status.trim();
    if status.is_empty() || status.len() > 32 {
        return Err(ServiceError::new(StatusCode::BAD_REQUEST, "INVALID_STATUS"));
    }
    Ok(status.to_string())
}

#[cfg(test)]
mod tests {
    use super::allow_submission;
    use crate::models::content::ContactSubmission;
    use bson::{doc, DateTime};

    #[test]
    fn missing_kind_defaults_to_contact() {
        let form: ContactSubmission = serde_json::from_str(
            r#"{"name":"A","email":"a@b.c","subject":"Hi","message":"Hello"}"#,
        )
        .unwrap();
        assert_eq!(form.kind, "contact");
    }

    #[test]
    fn rate_limit_blocks_the_sixth_hit() {
        let ip = format!("contact-test-{}", std::process::id());
        for _ in 0..5 {
            assert!(allow_submission(&ip));
        }
        assert!(!allow_submission(&ip));
    }

    #[test]
    fn missing_status_defaults_to_new() {
        let record: super::ContactRecord = bson::from_document(doc! {
            "name": "A",
            "email": "a@b.c",
            "subject": "Hi",
            "message": "Hello",
            "kind": "booking",
            "createdAt": DateTime::now(),
        })
        .unwrap();
        assert_eq!(record.status, "new");
        assert!(record.preferred_at.is_empty());
        assert!(record.id.is_none());
    }

    #[test]
    fn preferred_time_is_optional_and_must_parse() {
        assert!(matches!(
            super::clean_preferred("  ").as_deref(),
            Ok("")
        ));
        assert!(matches!(
            super::clean_preferred("2026-11-02T15:45").as_deref(),
            Ok("2026-11-02T15:45")
        ));
        assert!(super::clean_preferred("tomorrow").is_err());
        assert!(super::clean_preferred(&"x".repeat(41)).is_err());
    }

    #[test]
    fn clean_status_rejects_blank() {
        assert!(super::clean_status(" read ").is_ok());
        assert!(super::clean_status("  ").is_err());
        assert!(super::clean_status(&"x".repeat(33)).is_err());
    }
}
