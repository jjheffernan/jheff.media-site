use crate::services::media_feed_service;
use actix_web::{web, HttpResponse, Result};
use awc::Client;

pub async fn feed(
    client: web::Data<Client>,
    db: web::Data<mongodb::Database>,
    query: web::Query<FeedQuery>,
) -> Result<HttpResponse> {
    let limit = query.limit.unwrap_or(24);
    Ok(HttpResponse::Ok().json(
        media_feed_service::unified_feed(&client, &db, limit).await,
    ))
}

pub async fn instagram_featured() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(media_feed_service::instagram_featured()))
}

#[derive(serde::Deserialize)]
pub struct FeedQuery {
    pub limit: Option<usize>,
}
