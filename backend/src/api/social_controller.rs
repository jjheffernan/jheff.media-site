use crate::{services::social_service, utils::auth::require_admin};
use actix_web::{web, HttpRequest, HttpResponse, Result};
use awc::Client;
use mongodb::Database;

pub async fn hub(client: web::Data<Client>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(social_service::hub_config(&client).await))
}

pub async fn channels(
    req: HttpRequest,
    db: web::Data<Database>,
    client: web::Data<Client>,
) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    Ok(HttpResponse::Ok().json(social_service::channels(&client).await))
}
