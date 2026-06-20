use crate::services::social_service;
use actix_web::{web, HttpResponse, Result};
use awc::Client;

pub async fn hub(client: web::Data<Client>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(social_service::hub_config(&client).await))
}
