use crate::services::schedule_service;
use actix_web::{HttpResponse, Result};

pub async fn config() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(schedule_service::schedule_config()))
}
