use bson::doc;
use mongodb::Database;
use serde::{Deserialize, Serialize};
use std::env;

const COLLECTION: &str = "site_config";
const DOC_ID: &str = "main";

/// Demo URLs when provider is `static` and no JSON is set (env or DB).
pub const DEFAULT_STATIC_JSON: &str = r#"[{"id":"1","thumbnailUrl":"https://picsum.photos/seed/auto1/800/600","title":"Track still","mediaType":"image"},{"id":"2","thumbnailUrl":"https://picsum.photos/seed/auto2/800/600","title":"Garage detail","mediaType":"image"}]"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteConfigDoc {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(default = "default_provider")]
    pub feed_provider: String,
    #[serde(default)]
    pub immich_url: String,
    #[serde(default)]
    pub immich_api_key: String,
    #[serde(default)]
    pub immich_album_id: String,
    #[serde(default)]
    pub immich_shared_key: String,
    #[serde(default)]
    pub nextcloud_url: String,
    #[serde(default)]
    pub nextcloud_user: String,
    #[serde(default)]
    pub nextcloud_app_password: String,
    #[serde(default)]
    pub nextcloud_folder: String,
    #[serde(default)]
    pub feed_static_json: String,
    #[serde(default)]
    pub contact_email: String,
    #[serde(default)]
    pub public_url: String,
}

fn default_provider() -> String {
    String::from("none")
}

/// Admin GET shape — secrets masked.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteConfigView {
    pub feed_provider: String,
    pub immich_url: String,
    pub immich_api_key_set: bool,
    pub immich_api_key_hint: String,
    pub immich_album_id: String,
    pub immich_shared_key_set: bool,
    pub nextcloud_url: String,
    pub nextcloud_user: String,
    pub nextcloud_app_password_set: bool,
    pub nextcloud_app_password_hint: String,
    pub nextcloud_folder: String,
    pub feed_static_json: String,
    pub contact_email: String,
    pub public_url: String,
    pub from_db: bool,
    pub needs_setup: bool,
    /// True after a provider change — images may need `jheffmedia --restart`.
    pub restart_required: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteConfigUpdate {
    pub feed_provider: String,
    #[serde(default)]
    pub immich_url: String,
    /// Empty keeps the previous key.
    #[serde(default)]
    pub immich_api_key: String,
    #[serde(default)]
    pub immich_album_id: String,
    #[serde(default)]
    pub immich_shared_key: String,
    #[serde(default)]
    pub nextcloud_url: String,
    #[serde(default)]
    pub nextcloud_user: String,
    /// Empty keeps the previous password.
    #[serde(default)]
    pub nextcloud_app_password: String,
    #[serde(default)]
    pub nextcloud_folder: String,
    #[serde(default)]
    pub feed_static_json: String,
    #[serde(default)]
    pub contact_email: String,
    #[serde(default)]
    pub public_url: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub feed_provider: String,
    pub immich_url: String,
    pub immich_api_key: String,
    pub immich_album_id: String,
    pub immich_shared_key: String,
    pub nextcloud_url: String,
    pub nextcloud_user: String,
    pub nextcloud_app_password: String,
    pub nextcloud_folder: String,
    pub feed_static_json: String,
    pub contact_email: String,
    pub public_url: String,
    pub from_db: bool,
}

fn env_or(key: &str) -> String {
    env::var(key).unwrap_or_default()
}

fn from_env() -> SiteConfigDoc {
    SiteConfigDoc {
        id: DOC_ID.into(),
        feed_provider: env_or("YEW_FULLSTACK_FEED_PROVIDER")
            .to_lowercase()
            .if_empty("none"),
        immich_url: env_or("YEW_FULLSTACK_IMMICH_URL"),
        immich_api_key: env_or("YEW_FULLSTACK_IMMICH_API_KEY"),
        immich_album_id: env_or("YEW_FULLSTACK_IMMICH_ALBUM_ID"),
        immich_shared_key: env_or("YEW_FULLSTACK_IMMICH_SHARED_KEY"),
        nextcloud_url: env_or("YEW_FULLSTACK_NEXTCLOUD_URL"),
        nextcloud_user: env_or("YEW_FULLSTACK_NEXTCLOUD_USER"),
        nextcloud_app_password: env_or("YEW_FULLSTACK_NEXTCLOUD_APP_PASSWORD"),
        nextcloud_folder: env_or("YEW_FULLSTACK_NEXTCLOUD_FOLDER"),
        feed_static_json: env_or("YEW_FULLSTACK_FEED_STATIC_JSON"),
        contact_email: env_or("YEW_FULLSTACK_CONTACT_EMAIL"),
        public_url: env_or("YEW_FULLSTACK_PUBLIC_URL"),
    }
}

trait IfEmpty {
    fn if_empty(self, fallback: &str) -> String;
}

impl IfEmpty for String {
    fn if_empty(self, fallback: &str) -> String {
        if self.trim().is_empty() {
            fallback.into()
        } else {
            self
        }
    }
}

async fn load_doc(db: &Database) -> Option<SiteConfigDoc> {
    let coll = db.collection::<SiteConfigDoc>(COLLECTION);
    coll.find_one(doc! { "_id": DOC_ID }).await.ok().flatten()
}

pub async fn resolved(db: &Database) -> ResolvedConfig {
    if let Some(doc) = load_doc(db).await {
        return ResolvedConfig {
            feed_provider: normalize_provider(&doc.feed_provider),
            immich_url: doc.immich_url,
            immich_api_key: doc.immich_api_key,
            immich_album_id: doc.immich_album_id,
            immich_shared_key: doc.immich_shared_key,
            nextcloud_url: doc.nextcloud_url,
            nextcloud_user: doc.nextcloud_user,
            nextcloud_app_password: doc.nextcloud_app_password,
            nextcloud_folder: doc.nextcloud_folder,
            feed_static_json: doc.feed_static_json,
            contact_email: doc.contact_email,
            public_url: doc.public_url,
            from_db: true,
        };
    }
    let env = from_env();
    ResolvedConfig {
        feed_provider: normalize_provider(&env.feed_provider),
        immich_url: env.immich_url,
        immich_api_key: env.immich_api_key,
        immich_album_id: env.immich_album_id,
        immich_shared_key: env.immich_shared_key,
        nextcloud_url: env.nextcloud_url,
        nextcloud_user: env.nextcloud_user,
        nextcloud_app_password: env.nextcloud_app_password,
        nextcloud_folder: env.nextcloud_folder,
        feed_static_json: env.feed_static_json,
        contact_email: env.contact_email,
        public_url: env.public_url,
        from_db: false,
    }
}

fn normalize_provider(raw: &str) -> String {
    match raw.trim().to_lowercase().as_str() {
        "immich" => "immich".into(),
        "static" => "static".into(),
        "nextcloud" => "nextcloud".into(),
        _ => "none".into(),
    }
}

fn mask_secret(value: &str) -> (bool, String) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return (false, String::new());
    }
    let hint = if trimmed.chars().count() <= 4 {
        "****".into()
    } else {
        format!("••••{}", trimmed.chars().rev().take(4).collect::<String>().chars().rev().collect::<String>())
    };
    (true, hint)
}

fn to_view(cfg: &ResolvedConfig, restart_required: bool) -> SiteConfigView {
    let (immich_set, immich_hint) = mask_secret(&cfg.immich_api_key);
    let (shared_set, _) = mask_secret(&cfg.immich_shared_key);
    let (nc_set, nc_hint) = mask_secret(&cfg.nextcloud_app_password);
    SiteConfigView {
        feed_provider: cfg.feed_provider.clone(),
        immich_url: cfg.immich_url.clone(),
        immich_api_key_set: immich_set,
        immich_api_key_hint: immich_hint,
        immich_album_id: cfg.immich_album_id.clone(),
        immich_shared_key_set: shared_set,
        nextcloud_url: cfg.nextcloud_url.clone(),
        nextcloud_user: cfg.nextcloud_user.clone(),
        nextcloud_app_password_set: nc_set,
        nextcloud_app_password_hint: nc_hint,
        nextcloud_folder: cfg.nextcloud_folder.clone(),
        feed_static_json: cfg.feed_static_json.clone(),
        contact_email: cfg.contact_email.clone(),
        public_url: cfg.public_url.clone(),
        from_db: cfg.from_db,
        needs_setup: !cfg.from_db,
        restart_required,
    }
}

pub async fn get_view(db: &Database) -> SiteConfigView {
    to_view(&resolved(db).await, false)
}

fn allowed_http_url(url: &str) -> bool {
    let url = url.trim();
    if url.is_empty() {
        return true;
    }
    if url.chars().count() > 300 || url.contains(char::is_whitespace) {
        return false;
    }
    let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    else {
        return false;
    };
    let host = rest.split('/').next().unwrap_or(rest);
    !host.is_empty() && !host.contains('@')
}

fn validate_update(body: &SiteConfigUpdate) -> Result<String, &'static str> {
    let provider = normalize_provider(&body.feed_provider);
    if !allowed_http_url(&body.immich_url) || !allowed_http_url(&body.nextcloud_url) {
        return Err("BAD_URL");
    }
    if !allowed_http_url(&body.public_url) {
        return Err("BAD_URL");
    }
    let email = body.contact_email.trim();
    if !email.is_empty() && (!email.contains('@') || email.chars().count() > 120) {
        return Err("BAD_EMAIL");
    }
    let folder = body.nextcloud_folder.trim();
    if folder.contains("..") || folder.contains('\\') {
        return Err("BAD_FOLDER");
    }
    if provider == "immich" && body.immich_url.trim().is_empty() {
        return Err("IMMICH_URL_REQUIRED");
    }
    if provider == "nextcloud"
        && (body.nextcloud_url.trim().is_empty()
            || body.nextcloud_user.trim().is_empty()
            || body.nextcloud_folder.trim().is_empty())
    {
        return Err("NEXTCLOUD_REQUIRED");
    }
    if provider == "static" {
        let json = body.feed_static_json.trim();
        if !json.is_empty() && serde_json::from_str::<serde_json::Value>(json).is_err() {
            return Err("BAD_STATIC_JSON");
        }
    }
    Ok(provider)
}

pub async fn update(db: &Database, body: SiteConfigUpdate) -> Result<SiteConfigView, &'static str> {
    let provider = validate_update(&body)?;
    let previous = resolved(db).await;
    let provider_changed = previous.feed_provider != provider;

    // Empty secret fields keep the previous value (UI never echoes secrets).
    let immich_api_key = if body.immich_api_key.trim().is_empty() {
        previous.immich_api_key
    } else {
        body.immich_api_key.trim().to_string()
    };
    let nextcloud_app_password = if body.nextcloud_app_password.trim().is_empty() {
        previous.nextcloud_app_password
    } else {
        body.nextcloud_app_password.trim().to_string()
    };
    let immich_shared_key = if body.immich_shared_key.trim().is_empty() {
        previous.immich_shared_key
    } else {
        body.immich_shared_key.trim().to_string()
    };

    let doc = SiteConfigDoc {
        id: DOC_ID.into(),
        feed_provider: provider.clone(),
        immich_url: body.immich_url.trim().trim_end_matches('/').to_string(),
        immich_api_key,
        immich_album_id: body.immich_album_id.trim().to_string(),
        immich_shared_key,
        nextcloud_url: body.nextcloud_url.trim().trim_end_matches('/').to_string(),
        nextcloud_user: body.nextcloud_user.trim().to_string(),
        nextcloud_app_password,
        nextcloud_folder: body.nextcloud_folder.trim().trim_matches('/').to_string(),
        feed_static_json: body.feed_static_json.trim().to_string(),
        contact_email: body.contact_email.trim().to_lowercase(),
        public_url: body.public_url.trim().trim_end_matches('/').to_string(),
    };

    if provider == "immich" && doc.immich_api_key.is_empty() {
        return Err("IMMICH_KEY_REQUIRED");
    }
    if provider == "nextcloud" && doc.nextcloud_app_password.is_empty() {
        return Err("NEXTCLOUD_PASSWORD_REQUIRED");
    }

    let coll = db.collection::<SiteConfigDoc>(COLLECTION);
    coll.replace_one(doc! { "_id": DOC_ID }, &doc)
        .upsert(true)
        .await
        .map_err(|_| "SAVE_FAILED")?;

    // Never log secrets.
    info!("site_config saved (provider={})", provider);

    Ok(to_view(
        &ResolvedConfig {
            feed_provider: doc.feed_provider,
            immich_url: doc.immich_url,
            immich_api_key: doc.immich_api_key,
            immich_album_id: doc.immich_album_id,
            immich_shared_key: doc.immich_shared_key,
            nextcloud_url: doc.nextcloud_url,
            nextcloud_user: doc.nextcloud_user,
            nextcloud_app_password: doc.nextcloud_app_password,
            nextcloud_folder: doc.nextcloud_folder,
            feed_static_json: doc.feed_static_json,
            contact_email: doc.contact_email,
            public_url: doc.public_url,
            from_db: true,
        },
        provider_changed,
    ))
}

pub fn immich_ready(cfg: &ResolvedConfig) -> bool {
    !cfg.immich_url.trim().is_empty() && !cfg.immich_api_key.trim().is_empty()
}

pub fn nextcloud_ready(cfg: &ResolvedConfig) -> bool {
    !cfg.nextcloud_url.trim().is_empty()
        && !cfg.nextcloud_user.trim().is_empty()
        && !cfg.nextcloud_app_password.is_empty()
        && !cfg.nextcloud_folder.trim().is_empty()
        && !cfg.nextcloud_folder.contains("..")
        && !cfg.nextcloud_folder.contains('\\')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_provider_maps_known_values() {
        assert_eq!(normalize_provider("IMMICH"), "immich");
        assert_eq!(normalize_provider("nope"), "none");
    }

    #[test]
    fn mask_secret_hides_all_but_last_four() {
        let (set, hint) = mask_secret("abcdefghij");
        assert!(set);
        assert_eq!(hint, "••••ghij");
        assert!(!mask_secret("").0);
    }

    #[test]
    fn validate_rejects_bad_urls_and_requires_provider_fields() {
        let mut body = SiteConfigUpdate {
            feed_provider: "immich".into(),
            immich_url: "not-a-url".into(),
            immich_api_key: "k".into(),
            immich_album_id: String::new(),
            immich_shared_key: String::new(),
            nextcloud_url: String::new(),
            nextcloud_user: String::new(),
            nextcloud_app_password: String::new(),
            nextcloud_folder: String::new(),
            feed_static_json: String::new(),
            contact_email: String::new(),
            public_url: String::new(),
        };
        assert_eq!(validate_update(&body), Err("BAD_URL"));
        body.immich_url = "https://photos.example.com".into();
        assert!(validate_update(&body).is_ok());
        body.feed_provider = "nextcloud".into();
        assert_eq!(validate_update(&body), Err("NEXTCLOUD_REQUIRED"));
    }
}
