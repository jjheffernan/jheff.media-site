use crate::{
    error::ServiceError,
    models::{content::ContentCollection, response::ResponseBody},
    services::content_service,
    utils::auth::require_admin,
};
use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use futures_util::StreamExt;
use mongodb::Database;

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
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut content_type = String::from("image/jpeg");
    let mut filename: Option<String> = None;
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
        if field_name == "file" && file_bytes.is_none() {
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
            "file" if file_bytes.is_none() => file_bytes = Some(bytes),
            "featured" => featured = checked_flag(&String::from_utf8_lossy(&bytes)),
            "setAsCover" => set_as_cover = checked_flag(&String::from_utf8_lossy(&bytes)),
            _ => {}
        }
    }

    let Some(bytes) = file_bytes else {
        return Ok(
            ServiceError::new(actix_web::http::StatusCode::BAD_REQUEST, "MISSING_FILE").response(),
        );
    };

    match content_service::upload_media(
        kind,
        &collection_id,
        &bytes,
        &content_type,
        filename.as_deref(),
        featured,
        set_as_cover,
        &db,
    )
    .await
    {
        Ok(media) => Ok(HttpResponse::Ok().json(ResponseBody::new("UPLOADED", media))),
        Err(msg) if msg == "COLLECTION_NOT_FOUND" => {
            Ok(ServiceError::new(actix_web::http::StatusCode::NOT_FOUND, &msg).response())
        }
        Err(msg) if msg.starts_with("FILE_TOO_LARGE") || msg == "UNSUPPORTED_MEDIA_TYPE" => {
            Ok(ServiceError::new(actix_web::http::StatusCode::BAD_REQUEST, &msg).response())
        }
        Err(msg) => Ok(
            ServiceError::new(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &msg).response(),
        ),
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
