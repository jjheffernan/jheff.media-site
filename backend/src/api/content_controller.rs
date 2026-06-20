use crate::services::content_service;
use actix_web::{HttpResponse, Result, web};

pub async fn list_galleries() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(content_service::list_galleries()))
}

pub async fn get_gallery(path: web::Path<String>) -> Result<HttpResponse> {
    let id = path.into_inner();
    match content_service::get_gallery(&id) {
        Some(collection) => Ok(HttpResponse::Ok().json(collection)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "message": "GALLERY_NOT_FOUND",
            "data": ""
        }))),
    }
}

pub async fn list_shoots() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(content_service::list_shoots()))
}

pub async fn get_shoot(path: web::Path<String>) -> Result<HttpResponse> {
    let id = path.into_inner();
    match content_service::get_shoot(&id) {
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
