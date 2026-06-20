use crate::{
    error::{ServiceError, ServiceResult},
    models::feed::{FeedItem, FeedResponse},
};
use actix_web::http::StatusCode;
use awc::Client;
use serde::Deserialize;
use std::env;

const DEFAULT_LIMIT: usize = 24;
const MAX_LIMIT: usize = 48;

#[derive(Debug, Clone)]
enum FeedProvider {
    Immich(ImmichConfig),
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

fn provider_from_env() -> FeedProvider {
    let kind = env::var("YEW_FULLSTACK_FEED_PROVIDER")
        .unwrap_or_else(|_| String::from("none"))
        .to_lowercase();

    match kind.as_str() {
        "immich" => {
            let base_url = env::var("YEW_FULLSTACK_IMMICH_URL").unwrap_or_default();
            let api_key = env::var("YEW_FULLSTACK_IMMICH_API_KEY").unwrap_or_default();
            if base_url.is_empty() {
                return FeedProvider::Disabled;
            }
            FeedProvider::Immich(ImmichConfig {
                base_url: base_url.trim_end_matches('/').to_string(),
                api_key,
                album_id: env::var("YEW_FULLSTACK_IMMICH_ALBUM_ID").ok(),
                shared_key: env::var("YEW_FULLSTACK_IMMICH_SHARED_KEY").ok(),
            })
        }
        "static" => {
            let json = env::var("YEW_FULLSTACK_FEED_STATIC_JSON").unwrap_or_default();
            if json.is_empty() {
                return FeedProvider::Disabled;
            }
            match serde_json::from_str::<Vec<FeedItem>>(&json) {
                Ok(items) => FeedProvider::Static(items),
                Err(err) => {
                    error!("Invalid YEW_FULLSTACK_FEED_STATIC_JSON: {}", err);
                    FeedProvider::Disabled
                }
            }
        }
        _ => FeedProvider::Disabled,
    }
}

pub async fn list_feed(client: &Client, limit: Option<usize>) -> ServiceResult<FeedResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let provider = provider_from_env();

    match provider {
        FeedProvider::Immich(config) => list_immich(client, &config, limit).await,
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
        .map_err(|err| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    if !response.status().is_success() {
        return Err(ServiceError::new(
            StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY),
            format!("Immich album request failed: {}", response.status()),
        ));
    }

    let album: ImmichAlbum = response
        .json()
        .await
        .map_err(|err| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

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
        .map_err(|err| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    if !response.status().is_success() {
        return Err(ServiceError::new(
            StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY),
            format!("Immich search request failed: {}", response.status()),
        ));
    }

    let result: ImmichSearchResult = response
        .json()
        .await
        .map_err(|err| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(result.assets.items)
}

pub async fn proxy_thumbnail(client: &Client, asset_id: &str) -> ServiceResult<(Vec<u8>, String)> {
    let provider = provider_from_env();
    let config = match provider {
        FeedProvider::Immich(config) => config,
        _ => {
            return Err(ServiceError::new(
                StatusCode::NOT_FOUND,
                "Feed provider does not proxy thumbnails",
            ));
        }
    };

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
        .map_err(|err| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    if !response.status().is_success() {
        return Err(ServiceError::new(
            StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY),
            format!("Immich thumbnail request failed: {}", response.status()),
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
        .map_err(|err| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .to_vec();

    Ok((bytes, content_type))
}
