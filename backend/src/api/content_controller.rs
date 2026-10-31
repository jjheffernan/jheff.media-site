use crate::services::{content_service, content_upload_service};
use actix_web::{web, HttpResponse, Result};
use awc::Client;
use mongodb::Database;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SearchParams {
    #[serde(default)]
    q: String,
}

pub async fn hub(db: web::Data<Database>, client: web::Data<Client>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(content_service::hub(&db, &client).await))
}

pub async fn search(query: web::Query<SearchParams>, db: web::Data<Database>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(content_service::search(&db, &query.q).await))
}

pub async fn list_galleries(db: web::Data<Database>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(content_service::list_galleries_merged(&db).await))
}

pub async fn get_gallery(path: web::Path<String>, db: web::Data<Database>) -> Result<HttpResponse> {
    let id = path.into_inner();
    match content_service::get_gallery_merged(&id, &db).await {
        Some(collection) => Ok(HttpResponse::Ok().json(collection)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "message": "GALLERY_NOT_FOUND",
            "data": ""
        }))),
    }
}

pub async fn list_shoots(db: web::Data<Database>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(content_service::list_shoots_merged(&db).await))
}

pub async fn get_shoot(path: web::Path<String>, db: web::Data<Database>) -> Result<HttpResponse> {
    let id = path.into_inner();
    match content_service::get_shoot_merged(&id, &db).await {
        Some(collection) => Ok(HttpResponse::Ok().json(collection)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "message": "SHOOT_NOT_FOUND",
            "data": ""
        }))),
    }
}

pub async fn booking() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(content_service::booking_config()))
}

pub async fn other_sites() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(content_service::other_sites()))
}

pub async fn serve_media(path: web::Path<String>) -> Result<HttpResponse> {
    let asset_id = path.into_inner();
    match content_upload_service::load_asset(&asset_id) {
        Some((bytes, content_type)) => {
            Ok(HttpResponse::Ok().content_type(content_type).body(bytes))
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "message": "MEDIA_NOT_FOUND",
            "data": ""
        }))),
    }
}
