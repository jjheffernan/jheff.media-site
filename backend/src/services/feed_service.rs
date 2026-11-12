use crate::{
    error::{ServiceError, ServiceResult},
    models::feed::{FeedItem, FeedResponse},
    services::{
        library_import_service,
        site_config_service::{self, ResolvedConfig, DEFAULT_STATIC_JSON},
    },
};
use actix_web::http::StatusCode;
use awc::Client;
use mongodb::Database;
use serde::Deserialize;

const DEFAULT_LIMIT: usize = 24;
const MAX_LIMIT: usize = 48;

#[derive(Debug, Clone)]
enum FeedProvider {
    Immich(ImmichConfig),
    Nextcloud,
    Static(Vec<FeedItem>),
    Disabled,
}

#[derive(Debug, Clone)]
struct ImmichConfig {
    base_url: String,
    api_key: String,
    album_id: Option<String>,
    shared_key: Option<String>,
}

fn provider_from_resolved(cfg: &ResolvedConfig) -> FeedProvider {
    match cfg.feed_provider.as_str() {
        "immich" => {
            if !site_config_service::immich_ready(cfg) {
                return FeedProvider::Disabled;
            }
            FeedProvider::Immich(ImmichConfig {
                base_url: cfg.immich_url.trim_end_matches('/').to_string(),
                api_key: cfg.immich_api_key.clone(),
                album_id: Some(cfg.immich_album_id.clone()).filter(|id| !id.trim().is_empty()),
                shared_key: Some(cfg.immich_shared_key.clone()).filter(|k| !k.trim().is_empty()),
            })
        }
        "nextcloud" => {
            if site_config_service::nextcloud_ready(cfg) {
                FeedProvider::Nextcloud
            } else {
                FeedProvider::Disabled
            }
        }
        "static" => {
            let json = if cfg.feed_static_json.trim().is_empty() {
                DEFAULT_STATIC_JSON
            } else {
                cfg.feed_static_json.as_str()
            };
            match serde_json::from_str::<Vec<FeedItem>>(json) {
                Ok(items) => FeedProvider::Static(items),
                Err(err) => {
                    error!("Invalid feed static JSON: {}", err);
                    FeedProvider::Disabled
                }
            }
        }
        _ => FeedProvider::Disabled,
    }
}

pub async fn list_feed(
    client: &Client,
    db: &Database,
    limit: Option<usize>,
) -> ServiceResult<FeedResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let provider = provider_from_resolved(&site_config_service::resolved(db).await);

    match provider {
        FeedProvider::Immich(config) => list_immich(client, &config, limit).await,
        FeedProvider::Nextcloud => list_nextcloud(client, db, limit).await,
        FeedProvider::Static(items) => Ok(FeedResponse {
            source: String::from("static"),
            items: items.into_iter().take(limit).collect(),
        }),
        FeedProvider::Disabled => Ok(FeedResponse {
            source: String::from("none"),
            items: vec![],
        }),
    }
}

async fn list_immich(
    client: &Client,
    config: &ImmichConfig,
    limit: usize,
) -> ServiceResult<FeedResponse> {
    let assets = if let Some(album_id) = &config.album_id {
        fetch_album_assets(client, config, album_id).await?
    } else {
        fetch_recent_assets(client, config, limit).await?
    };

    let items = assets
        .into_iter()
        .take(limit)
        .map(|asset| {
            let thumbnail_url = format!("/api/feed/thumbnail/{}", asset.id);
            FeedItem {
                id: asset.id,
                title: asset.original_file_name,
                thumbnail_url,
                media_url: None,
                media_type: None,
                width: asset.exif_image_width,
                height: asset.exif_image_height,
            }
        })
        .collect();

    Ok(FeedResponse {
        source: String::from("immich"),
        items,
    })
}

async fn list_nextcloud(
    client: &Client,
    db: &Database,
    limit: usize,
) -> ServiceResult<FeedResponse> {
    let candidates = library_import_service::list_candidates(client, db, "nextcloud")
        .await
        .map_err(|code| ServiceError::new(StatusCode::BAD_GATEWAY, code))?;
    let items = candidates
        .into_iter()
        .take(limit)
        .map(|item| FeedItem {
            id: item.id.clone(),
            title: Some(item.name),
            thumbnail_url: format!("/api/feed/thumbnail/{}", item.id),
            media_url: None,
            media_type: Some(String::from("image")),
            width: None,
            height: None,
        })
        .collect();
    Ok(FeedResponse {
        source: String::from("nextcloud"),
        items,
    })
}

#[derive(Debug, Deserialize)]
struct ImmichAsset {
    id: String,
    #[serde(rename = "originalFileName")]
    original_file_name: Option<String>,
    #[serde(rename = "exifImageWidth")]
    exif_image_width: Option<u32>,
    #[serde(rename = "exifImageHeight")]
    exif_image_height: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ImmichAlbum {
    assets: Vec<ImmichAsset>,
}

#[derive(Debug, Deserialize)]
struct ImmichSearchResult {
    assets: ImmichSearchAssets,
}

#[derive(Debug, Deserialize)]
struct ImmichSearchAssets {
    items: Vec<ImmichAsset>,
}

fn upstream_err(context: &str, detail: impl std::fmt::Display) -> ServiceError {
    error!("{context}: {detail}");
    ServiceError::new(StatusCode::BAD_GATEWAY, "UPSTREAM")
}

async fn fetch_album_assets(
    client: &Client,
    config: &ImmichConfig,
    album_id: &str,
) -> ServiceResult<Vec<ImmichAsset>> {
    let url = format!("{}/api/albums/{}", config.base_url, album_id);
    let mut request = client.get(&url);
    if !config.api_key.is_empty() {
        request = request.append_header(("x-api-key", config.api_key.as_str()));
    }

    let mut response = request
        .send()
        .await
        .map_err(|err| upstream_err("Immich album request", err))?;

    if !response.status().is_success() {
        return Err(upstream_err(
            "Immich album request failed",
            response.status(),
        ));
    }

    let album: ImmichAlbum = response
        .json()
        .await
        .map_err(|err| upstream_err("Immich album decode", err))?;

    Ok(album.assets)
}

async fn fetch_recent_assets(
    client: &Client,
    config: &ImmichConfig,
    limit: usize,
) -> ServiceResult<Vec<ImmichAsset>> {
    let url = format!("{}/api/search/metadata", config.base_url);
    let body = serde_json::json!({
        "size": limit,
        "type": "IMAGE",
    });

    let mut request = client.post(&url);
    if !config.api_key.is_empty() {
        request = request.append_header(("x-api-key", config.api_key.as_str()));
    }

    let mut response = request
        .send_json(&body)
        .await
        .map_err(|err| upstream_err("Immich search request", err))?;

    if !response.status().is_success() {
        return Err(upstream_err(
            "Immich search request failed",
            response.status(),
        ));
    }

    let result: ImmichSearchResult = response
        .json()
        .await
        .map_err(|err| upstream_err("Immich search decode", err))?;

    Ok(result.assets.items)
}

pub async fn proxy_thumbnail(
    client: &Client,
    db: &Database,
    asset_id: &str,
) -> ServiceResult<(Vec<u8>, String)> {
    let provider = provider_from_resolved(&site_config_service::resolved(db).await);
    match provider {
        FeedProvider::Immich(config) => proxy_immich_thumbnail(client, &config, asset_id).await,
        FeedProvider::Nextcloud => {
            if !library_import_service::valid_asset_id("nextcloud", asset_id) {
                return Err(ServiceError::new(StatusCode::BAD_REQUEST, "BAD_ID"));
            }
            library_import_service::fetch_nextcloud_bytes(client, db, asset_id)
                .await
                .map_err(|code| {
                    let status = match code {
                        "BAD_ID" | "UNKNOWN_SOURCE" => StatusCode::BAD_REQUEST,
                        "NOT_CONFIGURED" => StatusCode::NOT_FOUND,
                        "FILE_TOO_LARGE" => StatusCode::PAYLOAD_TOO_LARGE,
                        _ => StatusCode::BAD_GATEWAY,
                    };
                    ServiceError::new(status, code)
                })
        }
        _ => Err(ServiceError::new(
            StatusCode::NOT_FOUND,
            "Feed provider does not proxy thumbnails",
        )),
    }
}

/// Public proxy must not fetch arbitrary Immich assets with the server API key.
/// ponytail: re-lists album/recent per thumb; add short TTL cache if Immich rate-limits.
async fn immich_feed_allows_asset(
    client: &Client,
    config: &ImmichConfig,
    asset_id: &str,
) -> ServiceResult<bool> {
    let assets = if let Some(album_id) = &config.album_id {
        fetch_album_assets(client, config, album_id).await?
    } else {
        // Same window as list_feed when no album is configured.
        fetch_recent_assets(client, config, MAX_LIMIT).await?
    };
    Ok(assets.iter().any(|asset| asset.id == asset_id))
}

async fn proxy_immich_thumbnail(
    client: &Client,
    config: &ImmichConfig,
    asset_id: &str,
) -> ServiceResult<(Vec<u8>, String)> {
    if !library_import_service::valid_asset_id("immich", asset_id) {
        return Err(ServiceError::new(StatusCode::BAD_REQUEST, "BAD_ID"));
    }
    if !immich_feed_allows_asset(client, config, asset_id).await? {
        return Err(ServiceError::new(StatusCode::NOT_FOUND, "NOT_FOUND"));
    }
    let mut url = format!(
        "{}/api/assets/{}/thumbnail?size=preview",
        config.base_url, asset_id
    );
    if let Some(key) = &config.shared_key {
        url = format!("{}&key={}", url, key);
    }

    let mut request = client.get(&url);
    if !config.api_key.is_empty() {
        request = request.append_header(("x-api-key", config.api_key.as_str()));
    }

    let mut response = request
        .send()
        .await
        .map_err(|err| upstream_err("Immich thumbnail request", err))?;

    if !response.status().is_success() {
        return Err(upstream_err(
            "Immich thumbnail request failed",
            response.status(),
        ));
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();

    let bytes = response
        .body()
        .await
        .map_err(|err| upstream_err("Immich thumbnail body", err))?
        .to_vec();

    Ok((bytes, content_type))
}
