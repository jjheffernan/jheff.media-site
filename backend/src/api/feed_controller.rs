use crate::services::feed_service;
use actix_web::{web, HttpResponse, Result};
use awc::Client;

pub async fn list(
    client: web::Data<Client>,
    query: web::Query<ListQuery>,
) -> Result<HttpResponse> {
    match feed_service::list_feed(&client, query.limit).await {
        Ok(feed) => Ok(HttpResponse::Ok().json(feed)),
        Err(err) => Ok(err.response()),
    }
}

pub async fn thumbnail(
    client: web::Data<Client>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let asset_id = path.into_inner();
    match feed_service::proxy_thumbnail(&client, &asset_id).await {
        Ok((bytes, content_type)) => Ok(HttpResponse::Ok()
            .content_type(content_type)
            .body(bytes)),
        Err(err) => Ok(err.response()),
    }
}

#[derive(serde::Deserialize)]
pub struct ListQuery {
    pub limit: Option<usize>,
}
