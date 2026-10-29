use crate::{
    models::content::ContactSubmission, services::contact_service, utils::auth::require_admin,
};
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

pub async fn list(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    match contact_service::list_contacts(&db).await {
        Ok(items) => Ok(HttpResponse::Ok().json(items)),
        Err(err) => Ok(err.response()),
    }
}

pub async fn set_status(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<serde_json::Value>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    let status = body.get("status").and_then(|value| value.as_str()).unwrap_or("");
    match contact_service::set_status(&path.into_inner(), status, &db).await {
        Ok(()) => Ok(HttpResponse::Ok().json(serde_json::json!({"message": "UPDATED"}))),
        Err(err) => Ok(err.response()),
    }
}

pub async fn remove(
    req: HttpRequest,
    path: web::Path<String>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    match contact_service::delete_contact(&path.into_inner(), &db).await {
        Ok(()) => Ok(HttpResponse::Ok().json(serde_json::json!({"message": "REMOVED"}))),
        Err(err) => Ok(err.response()),
    }
}
