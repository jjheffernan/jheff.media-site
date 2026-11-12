use crate::{
    services::{site_config_service, social_service},
    utils::auth::require_admin,
};
use actix_web::{web, HttpRequest, HttpResponse, Result};
use awc::Client;
use mongodb::Database;

pub async fn hub(db: web::Data<Database>, client: web::Data<Client>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(social_service::hub_config(&db, &client).await))
}

pub async fn channels(
    req: HttpRequest,
    db: web::Data<Database>,
    client: web::Data<Client>,
) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    Ok(HttpResponse::Ok().json(social_service::admin_social(&db, &client).await))
}

pub async fn update_social(
    req: HttpRequest,
    body: web::Json<site_config_service::SocialConfigUpdate>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    match site_config_service::update_social(&db, body.into_inner()).await {
        Ok(view) => Ok(HttpResponse::Ok().json(view)),
        Err(code) => {
            let status = match code {
                "SAVE_FAILED" => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                _ => actix_web::http::StatusCode::BAD_REQUEST,
            };
            Ok(HttpResponse::build(status).json(serde_json::json!({
                "message": code,
                "data": ""
            })))
        }
    }
}
