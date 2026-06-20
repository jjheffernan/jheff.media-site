use crate::models::content::{
    ContentCollection, ContentCollectionSummary, ContentListResponse,
};
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
    let items = parse_collections("YEW_FULLSTACK_GALLERIES_JSON");
    ContentListResponse {
        items: items.iter().map(to_summary).collect(),
        source: if items.is_empty() {
            String::from("none")
        } else {
            String::from("config")
        },
    }
}

pub fn get_gallery(id: &str) -> Option<ContentCollection> {
    parse_collections("YEW_FULLSTACK_GALLERIES_JSON")
        .into_iter()
        .find(|c| c.id == id)
}

pub fn list_shoots() -> ContentListResponse {
    let items = parse_collections("YEW_FULLSTACK_SHOOTS_JSON");
    ContentListResponse {
        items: items.iter().map(to_summary).collect(),
        source: if items.is_empty() {
            String::from("none")
        } else {
            String::from("config")
        },
    }
}

pub fn get_shoot(id: &str) -> Option<ContentCollection> {
    parse_collections("YEW_FULLSTACK_SHOOTS_JSON")
        .into_iter()
        .find(|c| c.id == id)
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
