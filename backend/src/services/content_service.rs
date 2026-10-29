use crate::models::content::{
    ContentCollection, ContentCollectionSummary, ContentListResponse, MediaItem,
};
use crate::models::social::SocialHubResponse;
use crate::services::{content_store_service, content_upload_service, social_service};
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
        cover_url: collection
            .cover_url
            .clone()
            .or_else(|| collection.media.first().map(|m| m.thumbnail_url.clone())),
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
    if let Some(c) = get_gallery(id) {
        return Some(c);
    }
    content_store_service::list_by_kind("gallery", db)
        .await
        .into_iter()
        .find(|c| c.id == id)
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
    if let Some(c) = get_shoot(id) {
        return Some(c);
    }
    content_store_service::list_by_kind("shoot", db)
        .await
        .into_iter()
        .find(|c| c.id == id)
}

pub async fn create_collection(collection: ContentCollection, kind: &str, db: &Database) -> bool {
    let collection = ContentCollection {
        kind: Some(kind.to_string()),
        ..collection
    };
    content_store_service::insert(collection, db).await
}

pub async fn upload_media(
    kind: &str,
    collection_id: &str,
    bytes: &[u8],
    content_type: &str,
    original_filename: Option<&str>,
    db: &Database,
) -> Result<MediaItem, String> {
    if content_store_service::find_by_kind_and_id(kind, collection_id, db)
        .await
        .is_none()
    {
        return Err(String::from("COLLECTION_NOT_FOUND"));
    }

    let media = content_upload_service::save_asset(bytes, content_type, original_filename)?;
    if !content_store_service::append_media(kind, collection_id, media.clone(), db).await {
        return Err(String::from("MEDIA_UPDATE_FAILED"));
    }
    let _ = content_store_service::set_cover_url_if_empty(
        kind,
        collection_id,
        &media.thumbnail_url,
        db,
    )
    .await;
    Ok(media)
}

fn list_from_env(env_key: &str) -> ContentListResponse {
    list_from_items(parse_collections(env_key))
}

fn list_from_items(items: Vec<ContentCollection>) -> ContentListResponse {
    ContentListResponse {
        items: items.iter().map(to_summary).collect(),
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
