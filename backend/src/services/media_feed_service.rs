use crate::error::{ServiceError, ServiceResult};
use crate::models::media::{InstagramFeatured, MediaFeedItem, MediaFeedResponse};
use crate::services::{
    content_service, instagram_service, library_import_service, site_config_service,
    site_config_service::{ResolvedConfig, DEFAULT_STATIC_JSON},
    youtube_service,
};
use actix_web::http::StatusCode;
use awc::Client;
use mongodb::Database;
use serde::Deserialize;
use std::env;

const MAX_LIMIT: usize = 48;
const THUMBNAIL_PATH: &str = "/api/media/thumbnail";

/// Missing or blank means every source. Unknown values are `UNKNOWN_SOURCE`.
pub fn feed_source(raw: Option<&str>) -> Result<Option<&str>, ServiceError> {
    let Some(source) = raw.map(str::trim).filter(|source| !source.is_empty()) else {
        return Ok(None);
    };
    match source {
        "instagram" | "youtube" | "gallery" | "shoot" | "immich" | "static" | "nextcloud" => {
            Ok(Some(source))
        }
        _ => Err(ServiceError::new(StatusCode::BAD_REQUEST, "UNKNOWN_SOURCE")),
    }
}

pub async fn unified_feed(
    client: &Client,
    db: &Database,
    limit: usize,
    source: Option<&str>,
) -> MediaFeedResponse {
    let limit = limit.clamp(1, MAX_LIMIT);
    let mut items: Vec<MediaFeedItem> = vec![];
    let mut sources: Vec<String> = vec![];
    let include = |name: &str| source.is_none() || source == Some(name);
    let cfg = site_config_service::resolved(db).await;

    if include("instagram") {
        let (instagram_posts, ig_source) =
            instagram_service::instagram_posts_or_fallback(client, limit, &cfg).await;
        if !instagram_posts.is_empty() {
            sources.push(ig_source);
            for post in instagram_posts {
                items.push(MediaFeedItem {
                    id: post.id.clone(),
                    source: String::from("instagram"),
                    media_type: String::from("image"),
                    title: Some(post.title.clone()),
                    caption: None,
                    thumbnail_url: post
                        .thumbnail_url
                        .clone()
                        .unwrap_or_else(|| post.url.clone()),
                    media_url: None,
                    link_url: Some(post.url.clone()),
                    published_at: post.published_at.clone(),
                    collection_id: None,
                    collection_kind: None,
                    collection_title: None,
                    featured: is_featured_instagram(&post.url),
                });
            }
        }
    }

    if include("youtube") {
        if let Ok(youtube_posts) = youtube_service::fetch_channel_videos(client, limit, &cfg).await {
            if !youtube_posts.is_empty() {
                sources.push(String::from("youtube"));
                for post in youtube_posts {
                    items.push(MediaFeedItem {
                        id: post.id.clone(),
                        source: String::from("youtube"),
                        media_type: String::from("video"),
                        title: Some(post.title.clone()),
                        caption: None,
                        thumbnail_url: post.thumbnail_url.clone().unwrap_or_default(),
                        media_url: Some(post.url.clone()),
                        link_url: Some(post.url.clone()),
                        published_at: post.published_at.clone(),
                        collection_id: None,
                        collection_kind: None,
                        collection_title: None,
                        featured: false,
                    });
                }
            }
        }
    }

    if include("gallery") {
        let galleries = content_service::list_galleries_merged(db).await;
        if !galleries.items.is_empty() {
            sources.push(String::from("galleries"));
            for g in galleries.items {
                if let Some(collection) = content_service::get_gallery_merged(&g.id, db).await {
                    for media in collection.media {
                        items.push(media_to_feed_item(
                            &media.id,
                            "gallery",
                            &collection.id,
                            &collection.title,
                            &media,
                        ));
                    }
                }
            }
        }
    }

    if include("shoot") {
        let shoots = content_service::list_shoots_merged(db).await;
        if !shoots.items.is_empty() {
            sources.push(String::from("shoots"));
            for s in shoots.items {
                if let Some(collection) = content_service::get_shoot_merged(&s.id, db).await {
                    for media in collection.media {
                        items.push(media_to_feed_item(
                            &media.id,
                            "shoot",
                            &collection.id,
                            &collection.title,
                            &media,
                        ));
                    }
                }
            }
        }
    }

    if let Ok(Some((provider_source, provider_items))) =
        provider_feed_items(client, db, &cfg, limit).await
    {
        if include(&provider_source) && !provider_items.is_empty() {
            sources.push(provider_source);
            items.extend(provider_items);
        }
    }

    items.sort_by(|a, b| {
        b.published_at
            .as_deref()
            .unwrap_or("")
            .cmp(a.published_at.as_deref().unwrap_or(""))
    });

    items.truncate(limit);

    if sources.is_empty() {
        sources.push(String::from("none"));
    }

    MediaFeedResponse { items, sources }
}

fn media_to_feed_item(
    id: &str,
    kind: &str,
    collection_id: &str,
    collection_title: &str,
    media: &crate::models::content::MediaItem,
) -> MediaFeedItem {
    let media_type = media
        .media_type
        .clone()
        .unwrap_or_else(|| String::from("image"));
    MediaFeedItem {
        id: format!("{}-{}", kind, id),
        source: String::from("upload"),
        media_type,
        title: media.title.clone(),
        caption: None,
        thumbnail_url: media.thumbnail_url.clone(),
        media_url: media.media_url.clone(),
        link_url: None,
        published_at: None,
        collection_id: Some(collection_id.to_string()),
        collection_kind: Some(kind.to_string()),
        collection_title: Some(collection_title.to_string()),
        featured: false,
    }
}

fn is_featured_instagram(url: &str) -> bool {
    let featured = env::var("YEW_FULLSTACK_INSTAGRAM_FEATURED_POST_URL").unwrap_or_default();
    if featured.is_empty() {
        return false;
    }
    url.contains(&featured) || featured.contains(url)
}

pub async fn instagram_featured(db: &Database) -> Option<InstagramFeatured> {
    let post_url = env::var("YEW_FULLSTACK_INSTAGRAM_FEATURED_POST_URL")
        .ok()
        .filter(|u| !u.is_empty())?;
    let thumbnail = env::var("YEW_FULLSTACK_INSTAGRAM_FEATURED_THUMBNAIL")
        .ok()
        .filter(|u| !u.is_empty());
    let title = env::var("YEW_FULLSTACK_INSTAGRAM_FEATURED_TITLE")
        .ok()
        .filter(|t| !t.is_empty())
        .or_else(|| Some(String::from("Latest from @jheffmedia")));
    let cfg = site_config_service::resolved(db).await;

    Some(InstagramFeatured {
        post_url,
        thumbnail_url: thumbnail,
        title,
        profile_url: instagram_service::instagram_profile_url(&cfg),
        handle: instagram_service::instagram_handle(&cfg),
    })
}

/// Proxies Immich/Nextcloud preview bytes for `<img src>` — browser never sees the API key.
pub async fn proxy_thumbnail(
    client: &Client,
    db: &Database,
    asset_id: &str,
) -> ServiceResult<(Vec<u8>, String)> {
    let cfg = site_config_service::resolved(db).await;
    match provider_from_resolved(&cfg) {
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

#[derive(Debug, Clone)]
enum FeedProvider {
    Immich(ImmichConfig),
    Nextcloud,
    Static(Vec<StaticFeedItem>),
    Disabled,
}

#[derive(Debug, Clone)]
struct ImmichConfig {
    base_url: String,
    api_key: String,
    album_id: Option<String>,
    shared_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StaticFeedItem {
    id: String,
    title: Option<String>,
    thumbnail_url: String,
    media_url: Option<String>,
    media_type: Option<String>,
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
            match serde_json::from_str::<Vec<StaticFeedItem>>(json) {
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

async fn provider_feed_items(
    client: &Client,
    db: &Database,
    cfg: &ResolvedConfig,
    limit: usize,
) -> ServiceResult<Option<(String, Vec<MediaFeedItem>)>> {
    match provider_from_resolved(cfg) {
        FeedProvider::Immich(config) => {
            let items = list_immich(client, &config, limit).await?;
            Ok(Some((String::from("immich"), items)))
        }
        FeedProvider::Nextcloud => {
            let items = list_nextcloud(client, db, limit).await?;
            Ok(Some((String::from("nextcloud"), items)))
        }
        FeedProvider::Static(items) => {
            let items = items
                .into_iter()
                .take(limit)
                .map(|item| MediaFeedItem {
                    id: item.id,
                    source: String::from("static"),
                    media_type: item
                        .media_type
                        .unwrap_or_else(|| String::from("image")),
                    title: item.title,
                    caption: None,
                    thumbnail_url: item.thumbnail_url,
                    media_url: item.media_url,
                    link_url: None,
                    published_at: None,
                    collection_id: None,
                    collection_kind: None,
                    collection_title: None,
                    featured: false,
                })
                .collect();
            Ok(Some((String::from("static"), items)))
        }
        FeedProvider::Disabled => Ok(None),
    }
}

async fn list_immich(
    client: &Client,
    config: &ImmichConfig,
    limit: usize,
) -> ServiceResult<Vec<MediaFeedItem>> {
    let assets = if let Some(album_id) = &config.album_id {
        fetch_album_assets(client, config, album_id).await?
    } else {
        fetch_recent_assets(client, config, limit).await?
    };

    Ok(assets
        .into_iter()
        .take(limit)
        .map(|asset| MediaFeedItem {
            id: asset.id.clone(),
            source: String::from("immich"),
            media_type: String::from("image"),
            title: asset.original_file_name,
            caption: None,
            thumbnail_url: format!("{}/{}", THUMBNAIL_PATH, asset.id),
            media_url: None,
            link_url: None,
            published_at: None,
            collection_id: None,
            collection_kind: None,
            collection_title: None,
            featured: false,
        })
        .collect())
}

async fn list_nextcloud(
    client: &Client,
    db: &Database,
    limit: usize,
) -> ServiceResult<Vec<MediaFeedItem>> {
    let candidates = library_import_service::list_candidates(client, db, "nextcloud")
        .await
        .map_err(|code| ServiceError::new(StatusCode::BAD_GATEWAY, code))?;
    Ok(candidates
        .into_iter()
        .take(limit)
        .map(|item| MediaFeedItem {
            id: item.id.clone(),
            source: String::from("nextcloud"),
            media_type: String::from("image"),
            title: Some(item.name),
            caption: None,
            thumbnail_url: format!("{}/{}", THUMBNAIL_PATH, item.id),
            media_url: None,
            link_url: None,
            published_at: None,
            collection_id: None,
            collection_kind: None,
            collection_title: None,
            featured: false,
        })
        .collect())
}

#[derive(Debug, Deserialize)]
struct ImmichAsset {
    id: String,
    #[serde(rename = "originalFileName")]
    original_file_name: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::feed_source;
    use actix_web::http::StatusCode;

    #[test]
    fn blank_source_is_all_and_unknown_is_rejected() {
        for (raw, expected) in [
            (None, None),
            (Some(""), None),
            (Some("  "), None),
            (Some("instagram"), Some("instagram")),
            (Some("youtube"), Some("youtube")),
            (Some("gallery"), Some("gallery")),
            (Some("shoot"), Some("shoot")),
            (Some("immich"), Some("immich")),
            (Some("static"), Some("static")),
            (Some("nextcloud"), Some("nextcloud")),
        ] {
            match feed_source(raw) {
                Ok(source) => assert_eq!(source, expected),
                Err(_) => panic!("rejected {raw:?}"),
            }
        }
        match feed_source(Some("tiktok")) {
            Ok(_) => panic!("tiktok should be rejected"),
            Err(err) => {
                assert_eq!(err.http_status, StatusCode::BAD_REQUEST);
                assert_eq!(err.body.message, "UNKNOWN_SOURCE");
            }
        }
    }
}
