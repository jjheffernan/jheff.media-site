use crate::models::content::{
    ContentCollection, ContentCollectionSummary, ContentListResponse, MediaItem,
    UpdateContentCollection,
};
use crate::models::social::SocialHubResponse;
use crate::services::{content_store_service, content_upload_service, schedule_service, social_service};
use awc::Client;
use futures_util::future::join3;
use mongodb::Database;
use serde::Serialize;
use std::env;

const HUB_SOCIAL_LIMIT: usize = 3;
const HUB_SHOOT_LIMIT: usize = 2;
const HUB_GALLERY_LIMIT: usize = 2;

#[derive(Debug, Clone, Serialize)]
pub struct ContentHubResponse {
    pub social: SocialHubResponse,
    pub shoots: ContentListResponse,
    pub galleries: ContentListResponse,
}

pub async fn hub(db: &Database, client: &Client) -> ContentHubResponse {
    let (mut social, mut shoots, mut galleries) = join3(
        social_service::hub_config(client),
        list_shoots_merged(db),
        list_galleries_merged(db),
    )
    .await;
    social.posts.truncate(HUB_SOCIAL_LIMIT);
    shoots.items.truncate(HUB_SHOOT_LIMIT);
    galleries.items.truncate(HUB_GALLERY_LIMIT);
    ContentHubResponse {
        social,
        shoots,
        galleries,
    }
}

fn parse_collections(env_key: &str) -> Vec<ContentCollection> {
    let json = env::var(env_key).unwrap_or_default();
    if json.is_empty() {
        return vec![];
    }
    match serde_json::from_str::<Vec<ContentCollection>>(&json) {
        Ok(items) => items,
        Err(err) => {
            error!("Invalid {}: {}", env_key, err);
            vec![]
        }
    }
}

fn to_summary(collection: &ContentCollection) -> ContentCollectionSummary {
    let has_video = collection
        .media
        .iter()
        .any(|m| m.media_type.as_deref() == Some("video"));
    ContentCollectionSummary {
        id: collection.id.clone(),
        title: collection.title.clone(),
        summary: collection.summary.clone(),
        cover_url: collection.cover_url.clone().filter(|url| !url.is_empty()),
        status: collection.status.clone(),
        date: collection.date.clone(),
        location: collection.location.clone(),
        media_count: collection.media.len(),
        has_video,
    }
}

pub fn list_galleries() -> ContentListResponse {
    list_from_env("YEW_FULLSTACK_GALLERIES_JSON")
}

pub async fn list_galleries_merged(db: &Database) -> ContentListResponse {
    let env_items = parse_collections("YEW_FULLSTACK_GALLERIES_JSON");
    let mongo_items = content_store_service::list_by_kind("gallery", db).await;
    list_from_items(merge_collections(env_items, mongo_items))
}

pub fn get_gallery(id: &str) -> Option<ContentCollection> {
    parse_collections("YEW_FULLSTACK_GALLERIES_JSON")
        .into_iter()
        .find(|c| c.id == id)
}

pub async fn get_gallery_merged(id: &str, db: &Database) -> Option<ContentCollection> {
    let found = if let Some(c) = get_gallery(id) {
        Some(c)
    } else {
        content_store_service::list_by_kind("gallery", db)
            .await
            .into_iter()
            .find(|c| c.id == id)
    };
    found.filter(is_public_collection)
}

pub fn list_shoots() -> ContentListResponse {
    list_from_env("YEW_FULLSTACK_SHOOTS_JSON")
}

pub async fn list_shoots_merged(db: &Database) -> ContentListResponse {
    let env_items = parse_collections("YEW_FULLSTACK_SHOOTS_JSON");
    let mongo_items = content_store_service::list_by_kind("shoot", db).await;
    list_from_items(merge_collections(env_items, mongo_items))
}

pub fn get_shoot(id: &str) -> Option<ContentCollection> {
    parse_collections("YEW_FULLSTACK_SHOOTS_JSON")
        .into_iter()
        .find(|c| c.id == id)
}

pub async fn get_shoot_merged(id: &str, db: &Database) -> Option<ContentCollection> {
    let found = if let Some(c) = get_shoot(id) {
        Some(c)
    } else {
        content_store_service::list_by_kind("shoot", db)
            .await
            .into_iter()
            .find(|c| c.id == id)
    };
    found.filter(is_public_collection)
}

pub async fn create_collection(collection: ContentCollection, kind: &str, db: &Database) -> bool {
    let collection = ContentCollection {
        kind: Some(kind.to_string()),
        ..collection
    };
    content_store_service::insert(collection, db).await
}

fn env_key_for_kind(kind: &str) -> Option<&'static str> {
    match kind {
        "gallery" => Some("YEW_FULLSTACK_GALLERIES_JSON"),
        "shoot" => Some("YEW_FULLSTACK_SHOOTS_JSON"),
        _ => None,
    }
}

fn is_env_only_id(kind: &str, id: &str) -> bool {
    env_key_for_kind(kind)
        .map(|key| parse_collections(key).iter().any(|c| c.id == id))
        .unwrap_or(false)
}

/// Mongo collections only. Env JSON ids return `ENV_ONLY_COLLECTION`.
async fn require_mongo_collection(
    kind: &str,
    collection_id: &str,
    db: &Database,
) -> Result<ContentCollection, String> {
    if let Some(c) = content_store_service::find_by_kind_and_id(kind, collection_id, db).await {
        return Ok(c);
    }
    if is_env_only_id(kind, collection_id) {
        return Err(String::from("ENV_ONLY_COLLECTION"));
    }
    Err(String::from("COLLECTION_NOT_FOUND"))
}

pub async fn update_collection(
    kind: &str,
    collection_id: &str,
    patch: UpdateContentCollection,
    db: &Database,
) -> Result<(), String> {
    require_mongo_collection(kind, collection_id, db).await?;
    if content_store_service::update_metadata(kind, collection_id, patch, db).await {
        Ok(())
    } else {
        Err(String::from("UPDATE_FAILED"))
    }
}

pub async fn delete_collection(kind: &str, collection_id: &str, db: &Database) -> Result<(), String> {
    require_mongo_collection(kind, collection_id, db).await?;
    let Some(removed) = content_store_service::delete_collection(kind, collection_id, db).await
    else {
        return Err(String::from("DELETE_FAILED"));
    };
    for media in &removed.media {
        let _ = content_upload_service::delete_asset(&media.id);
    }
    Ok(())
}

pub async fn delete_media(
    kind: &str,
    collection_id: &str,
    asset_id: &str,
    db: &Database,
) -> Result<(), String> {
    require_mongo_collection(kind, collection_id, db).await?;
    let Some(_removed) =
        content_store_service::remove_media(kind, collection_id, asset_id, db).await
    else {
        return Err(String::from("MEDIA_NOT_FOUND"));
    };
    let _ = content_upload_service::delete_asset(asset_id);
    Ok(())
}

pub async fn reorder_media(
    kind: &str,
    collection_id: &str,
    media_ids: &[String],
    db: &Database,
) -> Result<(), String> {
    require_mongo_collection(kind, collection_id, db).await?;
    if content_store_service::reorder_media(kind, collection_id, media_ids, db).await {
        Ok(())
    } else {
        Err(String::from("REORDER_FAILED"))
    }
}

pub async fn upload_media(
    kind: &str,
    collection_id: &str,
    bytes: &[u8],
    content_type: &str,
    original_filename: Option<&str>,
    featured: bool,
    set_as_cover: bool,
    db: &Database,
) -> Result<MediaItem, String> {
    require_mongo_collection(kind, collection_id, db).await?;

    let mut media = content_upload_service::save_asset(bytes, content_type, original_filename)?;
    media.featured = featured;
    if !content_store_service::append_media(kind, collection_id, media.clone(), db).await {
        let _ = content_upload_service::delete_asset(&media.id);
        return Err(String::from("MEDIA_UPDATE_FAILED"));
    }
    if set_as_cover
        && !content_store_service::set_cover_url(kind, collection_id, &media.thumbnail_url, db)
            .await
    {
        return Err(String::from("COVER_UPDATE_FAILED"));
    }
    Ok(media)
}

fn list_from_env(env_key: &str) -> ContentListResponse {
    list_from_items(parse_collections(env_key))
}

pub fn is_public_collection(collection: &ContentCollection) -> bool {
    !matches!(
        collection
            .status
            .as_deref()
            .map(|status| status.trim().to_ascii_lowercase())
            .as_deref(),
        Some("draft")
    )
}

fn list_from_items(items: Vec<ContentCollection>) -> ContentListResponse {
    ContentListResponse {
        items: items
            .iter()
            .filter(|collection| is_public_collection(collection))
            .map(to_summary)
            .collect(),
        source: if items.is_empty() {
            String::from("none")
        } else {
            String::from("config")
        },
    }
}

fn merge_collections(
    env_items: Vec<ContentCollection>,
    mongo_items: Vec<ContentCollection>,
) -> Vec<ContentCollection> {
    let mut merged = env_items;
    for item in mongo_items {
        if !merged.iter().any(|e| e.id == item.id) {
            merged.push(item);
        }
    }
    merged
}

const SEARCH_LIMIT: usize = 24;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub kind: String,
    pub id: String,
    pub title: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub query: String,
    pub hits: Vec<SearchHit>,
}

pub async fn search(db: &Database, raw: &str) -> SearchResponse {
    let query: String = raw.trim().chars().take(80).collect();
    let needle = query.to_lowercase();
    if needle.chars().count() < 2 {
        return SearchResponse { query, hits: vec![] };
    }

    let mut hits = Vec::new();
    for collection in public_collections("gallery", "YEW_FULLSTACK_GALLERIES_JSON", db).await {
        push_collection_hits("gallery", &collection, &needle, &mut hits);
    }
    for collection in public_collections("shoot", "YEW_FULLSTACK_SHOOTS_JSON", db).await {
        push_collection_hits("shoot", &collection, &needle, &mut hits);
    }
    for event in schedule_service::schedule_config(db).await.events {
        let matched = [&event.title, event.location.as_deref().unwrap_or(""), &event.starts_at, &event.kind]
            .iter()
            .any(|value| contains(value, &needle));
        if matched {
            hits.push(SearchHit {
                kind: "event".into(),
                id: event.id,
                title: event.title,
                detail: event.location.or(Some(event.starts_at)),
            });
        }
    }
    hits.truncate(SEARCH_LIMIT);
    SearchResponse { query, hits }
}

async fn public_collections(kind: &str, env_key: &str, db: &Database) -> Vec<ContentCollection> {
    let env_items = parse_collections(env_key);
    let mongo_items = content_store_service::list_by_kind(kind, db).await;
    merge_collections(env_items, mongo_items)
        .into_iter()
        .filter(is_public_collection)
        .collect()
}

fn push_collection_hits(kind: &str, collection: &ContentCollection, needle: &str, hits: &mut Vec<SearchHit>) {
    if !is_public_collection(collection) {
        return;
    }
    let matched = [
        collection.title.as_str(),
        collection.summary.as_deref().unwrap_or(""),
        collection.location.as_deref().unwrap_or(""),
        collection.date.as_deref().unwrap_or(""),
    ]
    .iter()
    .any(|value| contains(value, needle));
    if matched {
        hits.push(SearchHit {
            kind: kind.into(),
            id: collection.id.clone(),
            title: collection.title.clone(),
            detail: collection.location.clone().or(collection.summary.clone()),
        });
        return;
    }
    let label = if kind == "gallery" { "Gallery" } else { "Shoot" };
    for media in &collection.media {
        let Some(title) = media.title.as_deref() else {
            continue;
        };
        if contains(title, needle) {
            hits.push(SearchHit {
                kind: kind.into(),
                id: collection.id.clone(),
                title: title.to_string(),
                detail: Some(format!("{label} · {}", collection.title)),
            });
        }
    }
}

fn contains(hay: &str, needle: &str) -> bool {
    hay.to_lowercase().contains(needle)
}

pub fn booking_config() -> crate::models::content::BookingConfig {
    let embed_url = env::var("YEW_FULLSTACK_BOOKING_EMBED_URL")
        .ok()
        .filter(|u| !u.is_empty());
    let contact_email = env::var("YEW_FULLSTACK_CONTACT_EMAIL")
        .ok()
        .filter(|u| !u.is_empty());

    let has_embed = embed_url.is_some();

    crate::models::content::BookingConfig {
        embed_url,
        contact_email,
        source: if has_embed {
            String::from("embed")
        } else {
            String::from("none")
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_leaves_cover_empty_until_set() {
        let collection = ContentCollection {
            id: "a".into(),
            title: "A".into(),
            kind: None,
            summary: None,
            cover_url: None,
            status: None,
            date: None,
            location: None,
            media: vec![MediaItem {
                id: "m".into(),
                title: None,
                thumbnail_url: "https://example.test/thumb".into(),
                media_url: None,
                media_type: Some("image".into()),
                width: None,
                height: None,
                featured: false,
            }],
        };
        assert!(to_summary(&collection).cover_url.is_none());
    }

    #[test]
    fn draft_collections_stay_off_the_public_site() {
        let draft = ContentCollection {
            id: "id".into(),
            title: "Title".into(),
            kind: None,
            summary: None,
            cover_url: None,
            status: Some("draft".into()),
            date: None,
            location: None,
            media: vec![],
        };
        assert!(!is_public_collection(&draft));
        let published = ContentCollection {
            status: Some("published".into()),
            ..draft.clone()
        };
        assert!(is_public_collection(&published));
        let open = ContentCollection {
            status: None,
            ..draft
        };
        assert!(is_public_collection(&open));
    }

    #[test]
    fn search_skips_drafts_and_matches_a_photo_title() {
        let draft = sample("draft-set", "Secret canyon", Some("draft"), None);
        let published = sample(
            "laguna",
            "Track Day — Laguna Seca",
            Some("published"),
            Some("Grid start"),
        );
        let mut hits = Vec::new();
        push_collection_hits("gallery", &draft, "secret", &mut hits);
        push_collection_hits("gallery", &published, "grid", &mut hits);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "Grid start");
        assert_eq!(hits[0].id, "laguna");
    }

    fn sample(id: &str, title: &str, status: Option<&str>, media_title: Option<&str>) -> ContentCollection {
        ContentCollection {
            id: id.into(),
            title: title.into(),
            kind: None,
            summary: None,
            cover_url: None,
            status: status.map(str::to_string),
            date: None,
            location: None,
            media: media_title
                .map(|title| MediaItem {
                    id: "m".into(),
                    title: Some(title.into()),
                    thumbnail_url: String::new(),
                    media_url: None,
                    media_type: Some("image".into()),
                    width: None,
                    height: None,
                    featured: false,
                })
                .into_iter()
                .collect(),
        }
    }
}

pub fn other_sites() -> crate::models::content::OtherSitesResponse {
    let json = env::var("YEW_FULLSTACK_OTHER_SITES_JSON").unwrap_or_default();
    if json.is_empty() {
        return crate::models::content::OtherSitesResponse {
            sites: vec![],
            source: String::from("none"),
        };
    }

    match serde_json::from_str::<Vec<crate::models::content::OtherSite>>(&json) {
        Ok(sites) => crate::models::content::OtherSitesResponse {
            sites,
            source: String::from("config"),
        },
        Err(err) => {
            error!("Invalid YEW_FULLSTACK_OTHER_SITES_JSON: {}", err);
            crate::models::content::OtherSitesResponse {
                sites: vec![],
                source: String::from("error"),
            }
        }
    }
}

