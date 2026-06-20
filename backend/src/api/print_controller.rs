use crate::{
    error::ServiceError,
    models::media::{PrintSelectionInput, PrintSelectionView},
    services::print_service,
    utils::auth::extract_auth_user,
};
use actix_web::{web, HttpRequest, HttpResponse, Result};
use mongodb::Database;

pub async fn list(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse> {
    match extract_auth_user(&req, &db).await {
        Some(auth) => {
            let items = print_service::list_for_user(&auth, &db).await;
            let views: Vec<PrintSelectionView> = items.into_iter().map(PrintSelectionView::from).collect();
            Ok(HttpResponse::Ok().json(views))
        }
        None => Ok(
            ServiceError::new(actix_web::http::StatusCode::UNAUTHORIZED, "UNAUTHORIZED").response(),
        ),
    }
}

pub async fn add(
    req: HttpRequest,
    body: web::Json<PrintSelectionInput>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    match extract_auth_user(&req, &db).await {
        Some(auth) => match print_service::add(&auth, body.0, &db).await {
            Ok(item) => Ok(HttpResponse::Ok().json(PrintSelectionView::from(item))),
            Err(err) => Ok(err.response()),
        },
        None => Ok(
            ServiceError::new(actix_web::http::StatusCode::UNAUTHORIZED, "UNAUTHORIZED").response(),
        ),
    }
}

pub async fn remove(
    req: HttpRequest,
    path: web::Path<String>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    match extract_auth_user(&req, &db).await {
        Some(auth) => match print_service::remove(&auth, &path.into_inner(), &db).await {
            Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({"message":"REMOVED"}))),
            Err(err) => Ok(err.response()),
        },
        None => Ok(
            ServiceError::new(actix_web::http::StatusCode::UNAUTHORIZED, "UNAUTHORIZED").response(),
        ),
    }
}
