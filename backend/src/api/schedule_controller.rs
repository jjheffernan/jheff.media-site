use crate::{
    error::ServiceError,
    models::schedule::CreateScheduleEvent,
    services::schedule_service,
    utils::auth::{extract_auth_user, is_admin},
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
    match extract_auth_user(&req, &db).await {
        Some(auth) => {
            if !is_admin(&auth) {
                return Ok(
                    ServiceError::new(actix_web::http::StatusCode::FORBIDDEN, "FORBIDDEN")
                        .response(),
                );
            }
            match schedule_service::create_event(body.0, &db).await {
                Ok(id) => Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "CREATED",
                    "data": { "id": id }
                }))),
                Err(err) => Ok(err.response()),
            }
        }
        None => Ok(
            ServiceError::new(actix_web::http::StatusCode::UNAUTHORIZED, "UNAUTHORIZED").response(),
        ),
    }
}

pub async fn delete_event(
    req: HttpRequest,
    path: web::Path<String>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    match extract_auth_user(&req, &db).await {
        Some(auth) => {
            if !is_admin(&auth) {
                return Ok(
                    ServiceError::new(actix_web::http::StatusCode::FORBIDDEN, "FORBIDDEN")
                        .response(),
                );
            }
            match schedule_service::delete_event(&path.into_inner(), &db).await {
                Ok(()) => Ok(HttpResponse::Ok().json(serde_json::json!({"message": "REMOVED"}))),
                Err(err) => Ok(err.response()),
            }
        }
        None => Ok(
            ServiceError::new(actix_web::http::StatusCode::UNAUTHORIZED, "UNAUTHORIZED").response(),
        ),
    }
}
