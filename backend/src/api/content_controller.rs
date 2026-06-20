use crate::services::content_service;
use actix_web::{web, HttpResponse, Result};
use mongodb::Database;

pub async fn list_galleries(db: web::Data<Database>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(content_service::list_galleries_merged(&db).await))
}

pub async fn get_gallery(
    path: web::Path<String>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
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

pub async fn get_shoot(
    path: web::Path<String>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
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
