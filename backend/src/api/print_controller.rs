use crate::{
    models::media::{PrintCheckoutInput, PrintOrderView, PrintSelectionInput, PrintSelectionView},
    services::print_service,
    utils::auth::require_auth,
};
use actix_web::{web, HttpRequest, HttpResponse, Result};
use mongodb::Database;

pub async fn list(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse> {
    let auth = match require_auth(&req, &db).await {
        Ok(auth) => auth,
        Err(resp) => return Ok(resp),
    };
    let items = print_service::list_for_user(&auth, &db).await;
    let views: Vec<PrintSelectionView> = items.into_iter().map(PrintSelectionView::from).collect();
    Ok(HttpResponse::Ok().json(views))
}

pub async fn add(
    req: HttpRequest,
    body: web::Json<PrintSelectionInput>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let auth = match require_auth(&req, &db).await {
        Ok(auth) => auth,
        Err(resp) => return Ok(resp),
    };
    match print_service::add(&auth, body.0, &db).await {
        Ok(item) => Ok(HttpResponse::Ok().json(PrintSelectionView::from(item))),
        Err(err) => Ok(err.response()),
    }
}

pub async fn remove(
    req: HttpRequest,
    path: web::Path<String>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let auth = match require_auth(&req, &db).await {
        Ok(auth) => auth,
        Err(resp) => return Ok(resp),
    };
    match print_service::remove(&auth, &path.into_inner(), &db).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({"message":"REMOVED"}))),
        Err(err) => Ok(err.response()),
    }
}

pub async fn list_orders(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse> {
    let auth = match require_auth(&req, &db).await {
        Ok(auth) => auth,
        Err(resp) => return Ok(resp),
    };
    let orders: Vec<PrintOrderView> = print_service::list_orders(&auth, &db).await;
    Ok(HttpResponse::Ok().json(orders))
}

pub async fn checkout(
    req: HttpRequest,
    body: web::Json<PrintCheckoutInput>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let auth = match require_auth(&req, &db).await {
        Ok(auth) => auth,
        Err(resp) => return Ok(resp),
    };
    match print_service::checkout(&auth, body.0, &db).await {
        Ok(order) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "PRINT_REQUESTED",
            "data": order
        }))),
        Err(err) => Ok(err.response()),
    }
}
