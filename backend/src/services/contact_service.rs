use crate::{
    error::{ServiceError, ServiceResult},
    models::content::ContactSubmission,
};
use actix_web::http::StatusCode;
use bson::{doc, DateTime};
use mongodb::Database;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const CONTACTS_COLLECTION: &str = "contacts";
const RATE_LIMIT: usize = 5;
const RATE_WINDOW: Duration = Duration::from_secs(10 * 60);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ContactRecord {
    name: String,
    email: String,
    subject: String,
    message: String,
    created_at: DateTime,
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
        name: form.name.trim().to_string(),
        email: form.email.trim().to_string(),
        subject: form.subject.trim().to_string(),
        message: form.message.trim().to_string(),
        created_at: DateTime::now(),
    };

    let collection = db.collection(CONTACTS_COLLECTION);
    collection
        .insert_one(doc! {
            "name": &record.name,
            "email": &record.email,
            "subject": &record.subject,
            "message": &record.message,
            "createdAt": record.created_at,
        })
        .await
        .map_err(|err| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    info!(
        "Contact form received from {} <{}>",
        record.name, record.email
    );

    super::mailer::notify_contact(
        &record.name,
        &record.email,
        &record.subject,
        &record.message,
    )
    .await;

    Ok(String::from(
        "Message received. We'll get back to you soon.",
    ))
}

#[cfg(test)]
mod tests {
    use super::allow_submission;

    #[test]
    fn rate_limit_blocks_the_sixth_hit() {
        let ip = format!("contact-test-{}", std::process::id());
        for _ in 0..5 {
            assert!(allow_submission(&ip));
        }
        assert!(!allow_submission(&ip));
    }
}
