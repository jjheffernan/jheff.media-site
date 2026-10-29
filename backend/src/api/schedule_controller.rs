use crate::{
    models::schedule::CreateScheduleEvent, services::schedule_service, utils::auth::require_admin,
};
use actix_web::{web, HttpRequest, HttpResponse, Result};
use mongodb::Database;

pub async fn config(db: web::Data<Database>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(schedule_service::schedule_config(&db).await))
}

pub async fn create_event(
    req: HttpRequest,
    body: web::Json<CreateScheduleEvent>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    match schedule_service::create_event(body.0, &db).await {
        Ok(id) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "CREATED",
            "data": { "id": id }
        }))),
        Err(err) => Ok(err.response()),
    }
}

pub async fn delete_event(
    req: HttpRequest,
    path: web::Path<String>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    match schedule_service::delete_event(&path.into_inner(), &db).await {
        Ok(()) => Ok(HttpResponse::Ok().json(serde_json::json!({"message": "REMOVED"}))),
        Err(err) => Ok(err.response()),
    }
}
