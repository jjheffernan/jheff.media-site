use crate::{models::content::ContactSubmission, services::contact_service};
use actix_web::{web, HttpRequest, HttpResponse, Result};
use mongodb::Database;

pub async fn submit(
    req: HttpRequest,
    form: web::Json<ContactSubmission>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let client_ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();
    match contact_service::submit_contact(form.0, &client_ip, &db).await {
        Ok(message) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "CONTACT_SUCCESS",
            "data": message
        }))),
        Err(err) => Ok(err.response()),
    }
}
