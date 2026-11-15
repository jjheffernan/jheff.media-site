use crate::config::server::ServerConfig;
use actix_web::{
    http::{header, StatusCode},
    web, Error, HttpRequest, HttpResponse,
};
use mime_guess;
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

pub async fn serve_static(
    req: HttpRequest,
    _: web::Bytes,
    config: web::Data<ServerConfig>,
) -> Result<HttpResponse, Error> {
    let mut req_path = req.uri().path();
    // ponytail: block path escape into the container FS (static root only)
    if req_path.contains("..") {
        req_path = "/index.html";
    }
    if req_path.is_empty() || req_path.ends_with('/') {
        req_path = "/index.html";
    }

    let mut loc = format!("{}{}", config.http_serve_static, req_path);

    let path_loc = loc.clone();
    let path = Path::new(path_loc.as_str());

    // default to index.html (SPA fallback)
    if !path.is_file() {
        loc = format!("{}/index.html", config.http_serve_static);
    }

    let meta = fs::metadata(&loc).map_err(Error::from)?;
    let len = meta.len();
    let etag = etag_for(&meta, len);
    let cache_control = cache_control_for(&loc);

    if if_none_match(&req, &etag) {
        return Ok(HttpResponse::NotModified()
            .insert_header((header::ETAG, etag))
            .insert_header((header::CACHE_CONTROL, cache_control))
            .finish());
    }

    let content = fs::read(&loc).map_err(Error::from)?;
    let content_type = mime_guess::from_path(&loc)
        .first_or_octet_stream()
        .essence_str()
        .to_string();

    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(content_type)
        .insert_header((header::ETAG, etag))
        .insert_header((header::CACHE_CONTROL, cache_control))
        .body(content))
}

/// Strong ETag from mtime + size (cheap; good enough for deploy-replaced static trees).
fn etag_for(meta: &fs::Metadata, len: u64) -> String {
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("\"{mtime:x}-{len:x}\"")
}

fn cache_control_for(path: &str) -> &'static str {
    let name = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if name.eq_ignore_ascii_case("index.html") {
        // SPA shell references fingerprinted assets — always revalidate.
        "no-cache"
    } else if is_fingerprinted_asset(name) {
        // Trunk content-hashed JS/WASM/CSS.
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=86400"
    }
}

/// Trunk-style `name-<hexhash>.ext` (hash length ≥ 8).
fn is_fingerprinted_asset(file_name: &str) -> bool {
    let stem = file_name.rsplit_once('.').map(|(s, _)| s).unwrap_or(file_name);
    let Some((prefix, hash)) = stem.rsplit_once('-') else {
        return false;
    };
    !prefix.is_empty()
        && hash.len() >= 8
        && hash.chars().all(|c| c.is_ascii_hexdigit())
}

fn if_none_match(req: &HttpRequest, etag: &str) -> bool {
    let Some(value) = req.headers().get(header::IF_NONE_MATCH) else {
        return false;
    };
    let Ok(raw) = value.to_str() else {
        return false;
    };
    raw.split(',').any(|part| {
        let part = part.trim();
        part == "*" || part == etag || part.strip_prefix("W/") == Some(etag)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprinted_trunk_assets() {
        assert!(is_fingerprinted_asset("index-a1b2c3d4e5.js"));
        assert!(is_fingerprinted_asset("jheffmedia-site-frontend-deadbeef.wasm"));
        assert!(is_fingerprinted_asset("app-0123456789abcdef.css"));
        assert!(!is_fingerprinted_asset("index.html"));
        assert!(!is_fingerprinted_asset("watermark.svg"));
        assert!(!is_fingerprinted_asset("app.css"));
    }

    #[test]
    fn cache_control_by_name() {
        assert_eq!(cache_control_for("/www/index.html"), "no-cache");
        assert_eq!(
            cache_control_for("/www/index-abcdef12.js"),
            "public, max-age=31536000, immutable"
        );
        assert_eq!(
            cache_control_for("/www/watermark.svg"),
            "public, max-age=86400"
        );
    }

    #[test]
    fn etag_format() {
        // Construct via temp file so Metadata is real.
        let dir = std::env::temp_dir();
        let path = dir.join(format!("jheff-etag-test-{}", std::process::id()));
        fs::write(&path, b"hi").unwrap();
        let meta = fs::metadata(&path).unwrap();
        let tag = etag_for(&meta, meta.len());
        assert!(tag.starts_with('"') && tag.ends_with('"'));
        assert!(tag.contains('-'));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn if_none_match_parses_list() {
        let etag = "\"abc-1\"";
        let req = actix_web::test::TestRequest::default()
            .insert_header((header::IF_NONE_MATCH, "\"other\", \"abc-1\""))
            .to_http_request();
        assert!(if_none_match(&req, etag));

        let miss = actix_web::test::TestRequest::default()
            .insert_header((header::IF_NONE_MATCH, "\"nope\""))
            .to_http_request();
        assert!(!if_none_match(&miss, etag));
    }
}
