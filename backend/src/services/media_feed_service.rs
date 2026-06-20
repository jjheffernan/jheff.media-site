use crate::models::media::{InstagramFeatured, MediaFeedItem, MediaFeedResponse};
use crate::services::{content_service, instagram_service, youtube_service};
use awc::Client;
use mongodb::Database;
use std::env;

pub async fn unified_feed(client: &Client, db: &Database, limit: usize) -> MediaFeedResponse {
    let limit = limit.clamp(1, 48);
    let mut items: Vec<MediaFeedItem> = vec![];
    let mut sources: Vec<String> = vec![];

    let (instagram_posts, ig_source) =
        instagram_service::instagram_posts_or_fallback(client, limit).await;
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

    if let Ok(youtube_posts) = youtube_service::fetch_channel_videos(client, limit).await {
        if !youtube_posts.is_empty() {
            sources.push(String::from("youtube"));
            for post in youtube_posts {
                items.push(MediaFeedItem {
                    id: post.id.clone(),
                    source: String::from("youtube"),
                    media_type: String::from("video"),
                    title: Some(post.title.clone()),
                    caption: None,
                    thumbnail_url: post
                        .thumbnail_url
                        .clone()
                        .unwrap_or_default(),
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

pub fn instagram_featured() -> InstagramFeatured {
    let post_url = env::var("YEW_FULLSTACK_INSTAGRAM_FEATURED_POST_URL")
        .ok()
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| String::from("https://www.instagram.com/p/example/"));
    let thumbnail = env::var("YEW_FULLSTACK_INSTAGRAM_FEATURED_THUMBNAIL")
        .ok()
        .filter(|u| !u.is_empty());
    let title = env::var("YEW_FULLSTACK_INSTAGRAM_FEATURED_TITLE")
        .ok()
        .filter(|t| !t.is_empty())
        .or_else(|| Some(String::from("Latest from @jheffmedia")));

    InstagramFeatured {
        post_url,
        thumbnail_url: thumbnail,
        title,
        profile_url: instagram_service::instagram_profile_url(),
        handle: instagram_service::instagram_handle(),
    }
}
