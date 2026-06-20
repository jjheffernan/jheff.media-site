use crate::{models::content::ContactSubmission, services::contact_service};
use actix_web::{web, HttpResponse, Result};
use mongodb::Database;

pub async fn submit(
    form: web::Json<ContactSubmission>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    match contact_service::submit_contact(form.0, &db).await {
        Ok(message) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "CONTACT_SUCCESS",
            "data": message
        }))),
        Err(err) => Ok(err.response()),
    }
}
