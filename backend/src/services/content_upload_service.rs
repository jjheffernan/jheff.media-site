use crate::models::content::MediaItem;
use mime_guess::from_path;
use std::env;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const DEFAULT_UPLOAD_DIR: &str = "uploads/content";
const MAX_BYTES: usize = 10 * 1024 * 1024;

pub fn upload_dir() -> PathBuf {
    env::var("YEW_FULLSTACK_CONTENT_UPLOAD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_UPLOAD_DIR))
}

pub fn ensure_upload_dir() -> std::io::Result<PathBuf> {
    let dir = upload_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn max_upload_bytes() -> usize {
    env::var("YEW_FULLSTACK_CONTENT_UPLOAD_MAX_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(MAX_BYTES)
}

pub fn is_allowed_image(content_type: &str) -> bool {
    matches!(
        content_type,
        "image/jpeg" | "image/png" | "image/webp" | "image/gif"
    )
}

pub fn extension_for_content_type(content_type: &str) -> &'static str {
    match content_type {
        "image/png" => ".png",
        "image/webp" => ".webp",
        "image/gif" => ".gif",
        _ => ".jpg",
    }
}

pub fn save_asset(
    bytes: &[u8],
    content_type: &str,
    original_filename: Option<&str>,
) -> Result<MediaItem, String> {
    if bytes.len() > max_upload_bytes() {
        return Err(format!("FILE_TOO_LARGE: max {} bytes", max_upload_bytes()));
    }
    if !is_allowed_image(content_type) {
        return Err(String::from("UNSUPPORTED_MEDIA_TYPE"));
    }

    let dir = ensure_upload_dir().map_err(|e| e.to_string())?;
    let asset_id = Uuid::new_v4().to_string();
    let ext = original_filename
        .and_then(|name| Path::new(name).extension())
        .and_then(|e| e.to_str())
        .filter(|e| matches!(*e, "jpg" | "jpeg" | "png" | "webp" | "gif"))
        .map(|e| format!(".{}", e))
        .unwrap_or_else(|| extension_for_content_type(content_type).to_string());

    let filename = format!("{asset_id}{ext}");
    let path = dir.join(&filename);
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;

    let title = original_filename.map(str::to_string);
    let url = format!("/api/content/media/{asset_id}");
    Ok(MediaItem {
        id: asset_id,
        title,
        thumbnail_url: url.clone(),
        media_url: Some(url),
        media_type: Some(content_type.to_string()),
        width: None,
        height: None,
        featured: false,
    })
}

pub fn load_asset(asset_id: &str) -> Option<(Vec<u8>, String)> {
    let path = find_asset_path(&upload_dir(), asset_id)?;
    let bytes = std::fs::read(&path).ok()?;
    let content_type = from_path(&path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();
    Some((bytes, content_type))
}

pub fn find_asset_path(dir: &Path, asset_id: &str) -> Option<PathBuf> {
    if asset_id.is_empty() || asset_id.contains('/') || asset_id.contains('\\') {
        return None;
    }
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(asset_id)
            && name.len() > asset_id.len()
            && name[asset_id.len()..].starts_with('.')
        {
            return Some(entry.path());
        }
    }
    None
}

/// Best-effort delete of an uploaded asset file under the content upload dir.
pub fn delete_asset(asset_id: &str) -> bool {
    let Some(path) = find_asset_path(&upload_dir(), asset_id) else {
        return false;
    };
    std::fs::remove_file(path).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::{Mutex, OnceLock};

    fn with_temp_dir<F: FnOnce(&Path)>(test_fn: F) {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let temp = std::env::temp_dir().join(format!(
            "jheff-upload-test-{}",
            Uuid::new_v4()
        ));
        fs::create_dir_all(&temp).unwrap();
        std::env::set_var("YEW_FULLSTACK_CONTENT_UPLOAD_DIR", temp.to_string_lossy().as_ref());
        test_fn(&temp);
        fs::remove_dir_all(temp).ok();
    }

    #[test]
    fn save_and_load_round_trip() {
        with_temp_dir(|_| {
            let item = save_asset(b"fake-jpeg", "image/jpeg", Some("car.jpg")).unwrap();
            let (bytes, content_type) = load_asset(&item.id).unwrap();
            assert_eq!(bytes, b"fake-jpeg");
            assert!(content_type.starts_with("image/"));
            assert!(!item.featured);
        });
    }

    #[test]
    fn find_asset_path_rejects_path_traversal() {
        assert!(find_asset_path(Path::new("/tmp"), "../etc/passwd").is_none());
    }
}
