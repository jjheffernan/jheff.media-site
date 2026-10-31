use crate::{
    services::{metrics, telemetry_service},
    utils::auth::require_admin,
};
use actix_web::{web, HttpRequest, HttpResponse, Result};
use mongodb::Database;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TelemetryBody {
    path: String,
}

pub async fn metrics_text() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(metrics::exposition())
}

pub async fn ingest(
    req: HttpRequest,
    body: web::Json<TelemetryBody>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let client_ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();
    match telemetry_service::ingest(&body.path, &client_ip, &db).await {
        Ok(()) => Ok(HttpResponse::NoContent().finish()),
        Err(err) => Ok(err.response()),
    }
}

pub async fn analytics(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    Ok(HttpResponse::Ok().json(telemetry_service::summary(&db).await))
}
