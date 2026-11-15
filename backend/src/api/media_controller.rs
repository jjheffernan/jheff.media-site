use crate::services::media_feed_service;
use actix_web::{web, HttpResponse, Result};
use awc::Client;

pub async fn feed(
    client: web::Data<Client>,
    db: web::Data<mongodb::Database>,
    query: web::Query<FeedQuery>,
) -> Result<HttpResponse> {
    let limit = query.limit.unwrap_or(24);
    let source = match media_feed_service::feed_source(query.source.as_deref()) {
        Ok(source) => source,
        Err(err) => return Ok(err.response()),
    };
    Ok(
        HttpResponse::Ok()
            .json(media_feed_service::unified_feed(&client, &db, limit, source).await),
    )
}

pub async fn instagram_featured(db: web::Data<mongodb::Database>) -> Result<HttpResponse> {
    match media_feed_service::instagram_featured(&db).await {
        Some(featured) => Ok(HttpResponse::Ok().json(featured)),
        None => Ok(HttpResponse::NoContent().finish()),
    }
}

pub async fn thumbnail(
    client: web::Data<Client>,
    db: web::Data<mongodb::Database>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let asset_id = path.into_inner();
    match media_feed_service::proxy_thumbnail(&client, &db, &asset_id).await {
        Ok((bytes, content_type)) => Ok(HttpResponse::Ok()
            .content_type(content_type)
            .body(bytes)),
        Err(err) => Ok(err.response()),
    }
}

#[derive(serde::Deserialize)]
pub struct FeedQuery {
    pub limit: Option<usize>,
    pub source: Option<String>,
}
