use crate::{
    error::ServiceError,
    models::content::ContentCollection,
    services::content_service,
    utils::auth::{extract_auth_user, is_admin},
};
use actix_web::{web, HttpRequest, HttpResponse, Result};
use mongodb::Database;

pub async fn create_gallery(
    req: HttpRequest,
    body: web::Json<ContentCollection>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    match extract_auth_user(&req, &db).await {
        Some(auth) => {
            if !is_admin(&auth) {
                return Ok(
                    ServiceError::new(actix_web::http::StatusCode::FORBIDDEN, "FORBIDDEN").response(),
                );
            }
            if content_service::create_collection(body.0, "gallery", &db).await {
                Ok(HttpResponse::Ok().json(serde_json::json!({"message":"CREATED"})))
            } else {
                Ok(ServiceError::new(
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "CREATE_FAILED",
                )
                    .response())
            }
        }
        None => Ok(
            ServiceError::new(actix_web::http::StatusCode::UNAUTHORIZED, "UNAUTHORIZED").response(),
        ),
    }
}

pub async fn create_shoot(
    req: HttpRequest,
    body: web::Json<ContentCollection>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    match extract_auth_user(&req, &db).await {
        Some(auth) => {
            if !is_admin(&auth) {
                return Ok(
                    ServiceError::new(actix_web::http::StatusCode::FORBIDDEN, "FORBIDDEN").response(),
                );
            }
            if content_service::create_collection(body.0, "shoot", &db).await {
                Ok(HttpResponse::Ok().json(serde_json::json!({"message":"CREATED"})))
            } else {
                Ok(ServiceError::new(
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "CREATE_FAILED",
                )
                    .response())
            }
        }
        None => Ok(
            ServiceError::new(actix_web::http::StatusCode::UNAUTHORIZED, "UNAUTHORIZED").response(),
        ),
    }
}
