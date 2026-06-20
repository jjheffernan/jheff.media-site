use crate::models::content::{
    ContentCollection, ContentCollectionSummary, ContentListResponse,
};
use crate::services::content_store_service;
use mongodb::Database;
use std::env;

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

pub async fn create_collection(
    collection: ContentCollection,
    kind: &str,
    db: &Database,
) -> bool {
    let collection = ContentCollection {
        kind: Some(kind.to_string()),
        ..collection
    };
    content_store_service::insert(collection, db).await
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
