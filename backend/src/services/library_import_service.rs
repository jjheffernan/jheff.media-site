use crate::services::{content_service, content_store_service, content_upload_service};
use awc::Client;
use mongodb::Database;
use serde::Serialize;
use std::env;

pub const MAX_LIST: usize = 12;
pub const MAX_IMPORT: usize = 6;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LibraryCandidate {
    pub id: String,
    pub name: String,
}

struct ImmichConfig {
    base_url: String,
    api_key: String,
    album_id: Option<String>,
}

struct NextcloudConfig {
    base_url: String,
    user: String,
    password: String,
    folder: String,
}

pub fn configured_sources() -> Vec<&'static str> {
    let mut sources = Vec::new();
    if immich_config().is_some() {
        sources.push("immich");
    }
    if nextcloud_config().is_some() {
        sources.push("nextcloud");
    }
    sources
}

pub fn validate_import(source: &str, kind: &str, collection_id: &str, asset_ids: &[String]) -> Result<(), &'static str> {
    if !matches!(source, "immich" | "nextcloud") {
        return Err("UNKNOWN_SOURCE");
    }
    if !matches!(kind, "gallery" | "shoot") {
        return Err("BAD_KIND");
    }
    if !valid_collection_id(collection_id) {
        return Err("BAD_COLLECTION");
    }
    if asset_ids.is_empty() {
        return Err("EMPTY_SELECTION");
    }
    if asset_ids.len() > MAX_IMPORT {
        return Err("TOO_MANY");
    }
    if asset_ids.iter().any(|id| !valid_asset_id(source, id)) {
        return Err("BAD_ID");
    }
    Ok(())
}

pub fn valid_collection_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 80
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'-' | b'_'))
}

pub fn valid_asset_id(source: &str, id: &str) -> bool {
    match source {
        "immich" => {
            let bytes = id.as_bytes();
            (32..=36).contains(&bytes.len())
                && bytes
                    .iter()
                    .all(|byte| byte.is_ascii_hexdigit() || *byte == b'-')
        }
        "nextcloud" => {
            let bytes = id.as_bytes();
            !bytes.is_empty()
                && bytes.len() <= 180
                && !id.contains('/')
                && !id.contains('\\')
                && !id.contains("..")
                && is_image_name(id)
                && bytes.iter().all(|byte| (0x20..0x7f).contains(byte) && !matches!(*byte, b'?' | b'#'))
        }
        _ => false,
    }
}

pub async fn list_candidates(client: &Client, source: &str) -> Result<Vec<LibraryCandidate>, &'static str> {
    match source {
        "immich" => {
            let config = immich_config().ok_or("NOT_CONFIGURED")?;
            list_immich(client, &config).await
        }
        "nextcloud" => {
            let config = nextcloud_config().ok_or("NOT_CONFIGURED")?;
            list_nextcloud(client, &config).await
        }
        _ => Err("UNKNOWN_SOURCE"),
    }
}

pub async fn import_selected(
    client: &Client,
    db: &Database,
    source: &str,
    kind: &str,
    collection_id: &str,
    asset_ids: &[String],
) -> Result<usize, &'static str> {
    validate_import(source, kind, collection_id, asset_ids)?;
    if !configured_sources().contains(&source) {
        return Err("NOT_CONFIGURED");
    }
    let collection = content_store_service::find_by_kind_and_id(kind, collection_id, db)
        .await
        .ok_or("COLLECTION_NOT_FOUND")?;
    if content_service::is_public_collection(&collection) {
        return Err("NOT_DRAFT");
    }

    let mut copied = 0;
    for id in asset_ids {
        let (bytes, content_type, name) = download(client, source, id).await?;
        content_service::upload_media(kind, collection_id, &bytes, &content_type, Some(&name), false, false, db)
            .await
            .map_err(upload_code)?;
        copied += 1;
    }
    Ok(copied)
}

fn upload_code(err: String) -> &'static str {
    if err.starts_with("FILE_TOO_LARGE") {
        "FILE_TOO_LARGE"
    } else if err == "UNSUPPORTED_MEDIA_TYPE" {
        "UNSUPPORTED_MEDIA_TYPE"
    } else if err == "ENV_ONLY_COLLECTION" {
        "ENV_ONLY_COLLECTION"
    } else {
        "IMPORT_FAILED"
    }
}

async fn download(
    client: &Client,
    source: &str,
    id: &str,
) -> Result<(Vec<u8>, String, String), &'static str> {
    match source {
        "immich" => {
            let config = immich_config().ok_or("NOT_CONFIGURED")?;
            let url = format!("{}/api/assets/{id}/original", config.base_url);
            let mut request = client.get(url);
            request = request.append_header(("x-api-key", config.api_key.as_str()));
            let (bytes, content_type) = read_limited(request).await?;
            Ok((bytes, content_type, format!("{id}.jpg")))
        }
        "nextcloud" => {
            let config = nextcloud_config().ok_or("NOT_CONFIGURED")?;
            let url = format!(
                "{}/remote.php/dav/files/{}/{}/{}",
                config.base_url,
                encode_segment(&config.user),
                encode_path(&config.folder),
                encode_segment(id)
            );
            let request = client
                .get(url)
                .append_header(("Authorization", basic_auth(&config.user, &config.password)));
            let (bytes, content_type) = read_limited(request).await?;
            let content_type = if content_upload_service::is_allowed_image(&content_type) {
                content_type
            } else {
                mime_from_name(id).to_string()
            };
            Ok((bytes, content_type, id.to_string()))
        }
        _ => Err("UNKNOWN_SOURCE"),
    }
}

async fn read_limited(request: awc::ClientRequest) -> Result<(Vec<u8>, String), &'static str> {
    let mut response = request.send().await.map_err(|_| "UPSTREAM")?;
    if !response.status().is_success() {
        return Err("UPSTREAM");
    }
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .split(';')
        .next()
        .unwrap_or("application/octet-stream")
        .trim()
        .to_string();
    let bytes = response
        .body()
        .limit(content_upload_service::max_upload_bytes())
        .await
        .map_err(|_| "FILE_TOO_LARGE")?
        .to_vec();
    Ok((bytes, content_type))
}

async fn list_immich(client: &Client, config: &ImmichConfig) -> Result<Vec<LibraryCandidate>, &'static str> {
    let assets = if let Some(album_id) = &config.album_id {
        let url = format!("{}/api/albums/{album_id}", config.base_url);
        let mut response = client
            .get(url)
            .append_header(("x-api-key", config.api_key.as_str()))
            .send()
            .await
            .map_err(|_| "UPSTREAM")?;
        if !response.status().is_success() {
            return Err("UPSTREAM");
        }
        let album: ImmichAlbum = response.json().await.map_err(|_| "UPSTREAM")?;
        album.assets
    } else {
        let url = format!("{}/api/search/metadata", config.base_url);
        let mut response = client
            .post(url)
            .append_header(("x-api-key", config.api_key.as_str()))
            .send_json(&serde_json::json!({ "size": MAX_LIST, "type": "IMAGE" }))
            .await
            .map_err(|_| "UPSTREAM")?;
        if !response.status().is_success() {
            return Err("UPSTREAM");
        }
        let result: ImmichSearch = response.json().await.map_err(|_| "UPSTREAM")?;
        result.assets.items
    };

    Ok(assets
        .into_iter()
        .filter(|asset| valid_asset_id("immich", &asset.id))
        .take(MAX_LIST)
        .map(|asset| LibraryCandidate {
            name: asset
                .original_file_name
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| asset.id.clone()),
            id: asset.id,
        })
        .collect())
}

async fn list_nextcloud(
    client: &Client,
    config: &NextcloudConfig,
) -> Result<Vec<LibraryCandidate>, &'static str> {
    let url = format!(
        "{}/remote.php/dav/files/{}/{}/",
        config.base_url,
        encode_segment(&config.user),
        encode_path(&config.folder)
    );
    let method = actix_web::http::Method::from_bytes(b"PROPFIND").map_err(|_| "UPSTREAM")?;
    let body = r#"<?xml version="1.0"?><d:propfind xmlns:d="DAV:"><d:prop><d:displayname/></d:prop></d:propfind>"#;
    let mut response = client
        .request(method, url)
        .append_header(("Authorization", basic_auth(&config.user, &config.password)))
        .append_header(("Depth", "1"))
        .send_body(body)
        .await
        .map_err(|_| "UPSTREAM")?;
    if !response.status().is_success() {
        return Err("UPSTREAM");
    }
    let xml = response.body().await.map_err(|_| "UPSTREAM")?;
    let xml = String::from_utf8_lossy(&xml);
    let mut items = Vec::new();
    for href in hrefs_from_propfind(&xml) {
        let name = href
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("")
            .to_string();
        let name = decode_segment(&name);
        if href.ends_with('/') || !valid_asset_id("nextcloud", &name) {
            continue;
        }
        items.push(LibraryCandidate {
            id: name.clone(),
            name,
        });
        if items.len() == MAX_LIST {
            break;
        }
    }
    Ok(items)
}

fn immich_config() -> Option<ImmichConfig> {
    let base_url = env::var("YEW_FULLSTACK_IMMICH_URL").unwrap_or_default();
    let api_key = env::var("YEW_FULLSTACK_IMMICH_API_KEY").unwrap_or_default();
    if base_url.trim().is_empty() || api_key.trim().is_empty() {
        return None;
    }
    let album_id = env::var("YEW_FULLSTACK_IMMICH_ALBUM_ID")
        .ok()
        .filter(|id| valid_asset_id("immich", id));
    Some(ImmichConfig {
        base_url: base_url.trim_end_matches('/').to_string(),
        api_key,
        album_id,
    })
}

fn nextcloud_config() -> Option<NextcloudConfig> {
    let base_url = env::var("YEW_FULLSTACK_NEXTCLOUD_URL").unwrap_or_default();
    let user = env::var("YEW_FULLSTACK_NEXTCLOUD_USER").unwrap_or_default();
    let password = env::var("YEW_FULLSTACK_NEXTCLOUD_APP_PASSWORD").unwrap_or_default();
    let folder = env::var("YEW_FULLSTACK_NEXTCLOUD_FOLDER").unwrap_or_default();
    if base_url.trim().is_empty()
        || user.trim().is_empty()
        || password.is_empty()
        || folder.trim().is_empty()
        || folder.contains("..")
        || folder.contains('\\')
    {
        return None;
    }
    Some(NextcloudConfig {
        base_url: base_url.trim_end_matches('/').to_string(),
        user,
        password,
        folder: folder.trim_matches('/').to_string(),
    })
}

fn basic_auth(user: &str, password: &str) -> String {
    let raw = format!("{user}:{password}");
    format!("Basic {}", data_encoding::BASE64.encode(raw.as_bytes()))
}

fn encode_path(path: &str) -> String {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .map(encode_segment)
        .collect::<Vec<_>>()
        .join("/")
}

fn encode_segment(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn decode_segment(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or(""), 16) {
                out.push(byte);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| value.to_string())
}

pub fn hrefs_from_propfind(xml: &str) -> Vec<String> {
    let bytes = xml.as_bytes();
    let mut items = Vec::new();
    let mut index = 0;
    while index + 5 < bytes.len() {
        if bytes[index..].len() >= 5 && bytes[index..index + 5].eq_ignore_ascii_case(b"href>") {
            let start = index + 5;
            let rest = &xml[start..];
            if let Some(end) = rest.find('<') {
                let href = rest[..end].trim();
                if !href.is_empty() {
                    items.push(href.to_string());
                }
                index = start + end;
                continue;
            }
        }
        index += 1;
    }
    items
}

fn is_image_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".png")
        || lower.ends_with(".webp")
        || lower.ends_with(".gif")
}

fn mime_from_name(name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else {
        "image/jpeg"
    }
}

#[derive(serde::Deserialize)]
struct ImmichAsset {
    id: String,
    #[serde(rename = "originalFileName")]
    original_file_name: Option<String>,
}

#[derive(serde::Deserialize)]
struct ImmichAlbum {
    assets: Vec<ImmichAsset>,
}

#[derive(serde::Deserialize)]
struct ImmichSearch {
    assets: ImmichSearchAssets,
}

#[derive(serde::Deserialize)]
struct ImmichSearchAssets {
    items: Vec<ImmichAsset>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_rules_reject_bulk_and_paths() {
        let id = "0123456789abcdef0123456789abcdef";
        assert!(validate_import("immich", "gallery", "track-day", &[id.into()]).is_ok());
        let many: Vec<String> = (0..7).map(|_| id.to_string()).collect();
        assert_eq!(
            validate_import("immich", "gallery", "track-day", &many),
            Err("TOO_MANY")
        );
        assert_eq!(
            validate_import("nextcloud", "shoot", "track-day", &["../secret.jpg".into()]),
            Err("BAD_ID")
        );
        assert_eq!(
            validate_import("dropbox", "gallery", "track-day", &[id.into()]),
            Err("UNKNOWN_SOURCE")
        );
        assert!(!valid_asset_id("nextcloud", "folder/pic.jpg"));
        assert!(valid_asset_id("nextcloud", "pic.jpg"));
    }

    #[test]
    fn propfind_keeps_file_names_only() {
        let xml = r#"<d:response><d:href>/remote.php/dav/files/jj/Photos/</d:href></d:response>
            <d:response><d:href>/remote.php/dav/files/jj/Photos/grid%20start.jpg</d:href></d:response>"#;
        let hrefs = hrefs_from_propfind(xml);
        assert_eq!(hrefs.len(), 2);
        let name = decode_segment(hrefs[1].rsplit('/').next().unwrap());
        assert_eq!(name, "grid start.jpg");
        assert!(valid_asset_id("nextcloud", &name));
    }
}
