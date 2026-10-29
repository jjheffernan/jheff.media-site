use crate::{
    error::ServiceError,
    models::{
        content::{ContentCollection, ReorderMediaRequest, UpdateContentCollection},
        response::ResponseBody,
    },
    services::{content_service, library_import_service},
    utils::auth::require_admin,
};
use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use awc::Client;
use futures_util::StreamExt;
use mongodb::Database;
use serde::Deserialize;

pub async fn create_gallery(
    req: HttpRequest,
    body: web::Json<ContentCollection>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    if content_service::create_collection(body.0, "gallery", &db).await {
        Ok(HttpResponse::Ok().json(serde_json::json!({"message":"CREATED"})))
    } else {
        Ok(ServiceError::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            "CREATE_FAILED",
        )
        .response())
    }
}

pub async fn create_shoot(
    req: HttpRequest,
    body: web::Json<ContentCollection>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    if content_service::create_collection(body.0, "shoot", &db).await {
        Ok(HttpResponse::Ok().json(serde_json::json!({"message":"CREATED"})))
    } else {
        Ok(ServiceError::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            "CREATE_FAILED",
        )
        .response())
    }
}

pub async fn update_gallery(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<UpdateContentCollection>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    update_collection(req, path, body, db, "gallery").await
}

pub async fn update_shoot(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<UpdateContentCollection>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    update_collection(req, path, body, db, "shoot").await
}

async fn update_collection(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<UpdateContentCollection>,
    db: web::Data<Database>,
    kind: &str,
) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    match content_service::update_collection(kind, &path.into_inner(), body.0, &db).await {
        Ok(()) => Ok(HttpResponse::Ok().json(serde_json::json!({"message":"UPDATED"}))),
        Err(msg) => Ok(mongo_op_error(&msg)),
    }
}

pub async fn delete_gallery(
    req: HttpRequest,
    path: web::Path<String>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    delete_collection(req, path, db, "gallery").await
}

pub async fn delete_shoot(
    req: HttpRequest,
    path: web::Path<String>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    delete_collection(req, path, db, "shoot").await
}

async fn delete_collection(
    req: HttpRequest,
    path: web::Path<String>,
    db: web::Data<Database>,
    kind: &str,
) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    match content_service::delete_collection(kind, &path.into_inner(), &db).await {
        Ok(()) => Ok(HttpResponse::Ok().json(serde_json::json!({"message":"REMOVED"}))),
        Err(msg) => Ok(mongo_op_error(&msg)),
    }
}

pub async fn delete_gallery_media(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    delete_media(req, path, db, "gallery").await
}

pub async fn delete_shoot_media(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    delete_media(req, path, db, "shoot").await
}

async fn delete_media(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    db: web::Data<Database>,
    kind: &str,
) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    let (collection_id, asset_id) = path.into_inner();
    match content_service::delete_media(kind, &collection_id, &asset_id, &db).await {
        Ok(()) => Ok(HttpResponse::Ok().json(serde_json::json!({"message":"REMOVED"}))),
        Err(msg) if msg == "MEDIA_NOT_FOUND" => {
            Ok(ServiceError::new(actix_web::http::StatusCode::NOT_FOUND, &msg).response())
        }
        Err(msg) => Ok(mongo_op_error(&msg)),
    }
}

pub async fn reorder_gallery_media(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<ReorderMediaRequest>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    reorder_media(req, path, body, db, "gallery").await
}

pub async fn reorder_shoot_media(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<ReorderMediaRequest>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    reorder_media(req, path, body, db, "shoot").await
}

async fn reorder_media(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<ReorderMediaRequest>,
    db: web::Data<Database>,
    kind: &str,
) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    match content_service::reorder_media(kind, &path.into_inner(), &body.media_ids, &db).await {
        Ok(()) => Ok(HttpResponse::Ok().json(serde_json::json!({"message":"REORDERED"}))),
        Err(msg) => Ok(mongo_op_error(&msg)),
    }
}

struct IncomingFile {
    bytes: Vec<u8>,
    content_type: String,
    filename: Option<String>,
}

async fn upload_collection_media(
    req: HttpRequest,
    path: web::Path<String>,
    mut payload: Multipart,
    db: web::Data<Database>,
    kind: &str,
) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }

    let collection_id = path.into_inner();
    let mut files: Vec<IncomingFile> = Vec::new();
    let mut featured = false;
    let mut set_as_cover = false;

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(err) => {
                error!("multipart read error: {err}");
                return Ok(ServiceError::new(
                    actix_web::http::StatusCode::BAD_REQUEST,
                    "INVALID_MULTIPART",
                )
                .response());
            }
        };

        let field_name = field
            .content_disposition()
            .and_then(|d| d.get_name().map(str::to_string))
            .unwrap_or_default();

        let mut content_type = String::from("image/jpeg");
        let mut filename: Option<String> = None;
        if field_name == "file" || field_name == "files" {
            if let Some(ct) = field.content_type().map(|m| m.to_string()) {
                content_type = ct;
            }
            if let Some(name) = field
                .content_disposition()
                .and_then(|d| d.get_filename().map(str::to_string))
            {
                filename = Some(name);
            }
        }

        let mut bytes = Vec::new();
        while let Some(chunk) = field.next().await {
            match chunk {
                Ok(data) => bytes.extend_from_slice(&data),
                Err(err) => {
                    error!("multipart chunk error: {err}");
                    return Ok(ServiceError::new(
                        actix_web::http::StatusCode::BAD_REQUEST,
                        "INVALID_MULTIPART",
                    )
                    .response());
                }
            }
        }

        match field_name.as_str() {
            "file" | "files" => files.push(IncomingFile {
                bytes,
                content_type,
                filename,
            }),
            "featured" => featured = checked_flag(&String::from_utf8_lossy(&bytes)),
            "setAsCover" => set_as_cover = checked_flag(&String::from_utf8_lossy(&bytes)),
            _ => {}
        }
    }

    if files.is_empty() {
        return Ok(
            ServiceError::new(actix_web::http::StatusCode::BAD_REQUEST, "MISSING_FILE").response(),
        );
    }

    let mut uploaded = Vec::with_capacity(files.len());
    for (i, file) in files.into_iter().enumerate() {
        // setAsCover only first file; featured applies to every file in the request
        let cover = set_as_cover && i == 0;
        match content_service::upload_media(
            kind,
            &collection_id,
            &file.bytes,
            &file.content_type,
            file.filename.as_deref(),
            featured,
            cover,
            &db,
        )
        .await
        {
            Ok(media) => uploaded.push(media),
            Err(msg) => return Ok(upload_error(&msg)),
        }
    }

    Ok(HttpResponse::Ok().json(ResponseBody::new("UPLOADED", uploaded)))
}

fn upload_error(msg: &str) -> HttpResponse {
    if msg == "COLLECTION_NOT_FOUND" || msg == "ENV_ONLY_COLLECTION" {
        ServiceError::new(actix_web::http::StatusCode::NOT_FOUND, msg).response()
    } else if msg.starts_with("FILE_TOO_LARGE") || msg == "UNSUPPORTED_MEDIA_TYPE" {
        ServiceError::new(actix_web::http::StatusCode::BAD_REQUEST, msg).response()
    } else {
        ServiceError::new(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, msg).response()
    }
}

fn mongo_op_error(msg: &str) -> HttpResponse {
    match msg {
        "COLLECTION_NOT_FOUND" | "ENV_ONLY_COLLECTION" => {
            ServiceError::new(actix_web::http::StatusCode::NOT_FOUND, msg).response()
        }
        _ => ServiceError::new(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, msg).response(),
    }
}

pub async fn upload_gallery_media(
    req: HttpRequest,
    path: web::Path<String>,
    payload: Multipart,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    upload_collection_media(req, path, payload, db, "gallery").await
}

fn checked_flag(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "on" | "yes"
    )
}

pub async fn upload_shoot_media(
    req: HttpRequest,
    path: web::Path<String>,
    payload: Multipart,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    upload_collection_media(req, path, payload, db, "shoot").await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryImportRequest {
    kind: String,
    collection_id: String,
    asset_ids: Vec<String>,
}

pub async fn library_sources(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    Ok(HttpResponse::Ok().json(ResponseBody::new(
        "OK",
        serde_json::json!({ "sources": library_import_service::configured_sources() }),
    )))
}

pub async fn library_list(
    req: HttpRequest,
    path: web::Path<String>,
    db: web::Data<Database>,
    client: web::Data<Client>,
) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    let source = path.into_inner();
    match library_import_service::list_candidates(&client, &source).await {
        Ok(items) => Ok(HttpResponse::Ok().json(ResponseBody::new("OK", items))),
        Err(code) => Ok(library_error(code)),
    }
}

pub async fn library_import(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<LibraryImportRequest>,
    db: web::Data<Database>,
    client: web::Data<Client>,
) -> Result<HttpResponse> {
    if let Err(resp) = require_admin(&req, &db).await {
        return Ok(resp);
    }
    let source = path.into_inner();
    match library_import_service::import_selected(
        &client,
        &db,
        &source,
        &body.kind,
        &body.collection_id,
        &body.asset_ids,
    )
    .await
    {
        Ok(copied) => Ok(HttpResponse::Ok().json(ResponseBody::new(
            "IMPORTED",
            serde_json::json!({ "copied": copied }),
        ))),
        Err(code) => Ok(library_error(code)),
    }
}

fn library_error(code: &str) -> HttpResponse {
    let status = match code {
        "UNKNOWN_SOURCE" | "BAD_KIND" | "BAD_COLLECTION" | "EMPTY_SELECTION" | "TOO_MANY"
        | "BAD_ID" | "NOT_DRAFT" | "FILE_TOO_LARGE" | "UNSUPPORTED_MEDIA_TYPE" => {
            actix_web::http::StatusCode::BAD_REQUEST
        }
        "NOT_CONFIGURED" | "COLLECTION_NOT_FOUND" | "ENV_ONLY_COLLECTION" => {
            actix_web::http::StatusCode::NOT_FOUND
        }
        _ => actix_web::http::StatusCode::BAD_GATEWAY,
    };
    ServiceError::new(status, code).response()
}

#[cfg(test)]
mod tests {
    use super::checked_flag;

    #[test]
    fn checked_flag_accepts_common_truthy_values() {
        assert!(checked_flag("true"));
        assert!(checked_flag(" on "));
        assert!(!checked_flag(""));
        assert!(!checked_flag("false"));
    }
}
