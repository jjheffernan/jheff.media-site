use crate::{
    error::{ServiceError, ServiceResult},
    models::content::ContactSubmission,
};
use actix_web::http::StatusCode;
use bson::{doc, DateTime};
use mongodb::Database;
use serde::Serialize;

const CONTACTS_COLLECTION: &str = "contacts";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ContactRecord {
    name: String,
    email: String,
    subject: String,
    message: String,
    created_at: DateTime,
}

pub async fn submit_contact(form: ContactSubmission, db: &Database) -> ServiceResult<String> {
    if form.name.trim().is_empty() || form.email.trim().is_empty() || form.message.trim().is_empty()
    {
        return Err(ServiceError::new(
            StatusCode::BAD_REQUEST,
            "Name, email, and message are required.",
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

    Ok(String::from("Message received. We'll get back to you soon."))
}
