use crate::{
    error::{ServiceError, ServiceResult},
    models::social::SocialPost,
};
use actix_web::http::StatusCode;
use awc::Client;
use serde::Deserialize;
use std::env;

const DEFAULT_CHANNEL_HANDLE: &str = "jheffmedia";

#[derive(Debug, Deserialize)]
struct YouTubeSearchResponse {
    items: Vec<YouTubeSearchItem>,
}

#[derive(Debug, Deserialize)]
struct YouTubeSearchItem {
    id: YouTubeVideoId,
    snippet: YouTubeSnippet,
}

#[derive(Debug, Deserialize)]
struct YouTubeVideoId {
    #[serde(rename = "videoId")]
    video_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YouTubeSnippet {
    title: String,
    description: Option<String>,
    published_at: Option<String>,
    thumbnails: YouTubeThumbnails,
}

#[derive(Debug, Deserialize)]
struct YouTubeThumbnails {
    high: Option<YouTubeThumb>,
    medium: Option<YouTubeThumb>,
    default: Option<YouTubeThumb>,
}

#[derive(Debug, Deserialize)]
struct YouTubeThumb {
    url: String,
}

pub fn channel_url() -> String {
    env::var("YEW_FULLSTACK_YOUTUBE_CHANNEL_URL")
        .ok()
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| format!("https://www.youtube.com/@{}", DEFAULT_CHANNEL_HANDLE))
}

pub async fn fetch_channel_videos(client: &Client, limit: usize) -> ServiceResult<Vec<SocialPost>> {
    let api_key = env::var("YEW_FULLSTACK_YOUTUBE_API_KEY")
        .ok()
        .filter(|k| !k.is_empty());

    if api_key.is_none() {
        return Ok(static_youtube_posts());
    }

    let channel_id = env::var("YEW_FULLSTACK_YOUTUBE_CHANNEL_ID")
        .ok()
        .filter(|id| !id.is_empty());

    let url = if let Some(channel_id) = channel_id {
        format!(
            "https://www.googleapis.com/youtube/v3/search?part=snippet&channelId={}&type=video&order=date&maxResults={}&key={}",
            channel_id,
            limit.clamp(1, 25),
            api_key.unwrap()
        )
    } else {
        format!(
            "https://www.googleapis.com/youtube/v3/search?part=snippet&q={}&type=channel&maxResults=1&key={}",
            DEFAULT_CHANNEL_HANDLE,
            api_key.unwrap()
        )
    };

    let mut response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !response.status().is_success() {
        warn!("YouTube API failed: {}", response.status());
        return Ok(static_youtube_posts());
    }

    let payload: YouTubeSearchResponse = response
        .json()
        .await
        .map_err(|e| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let posts: Vec<crate::models::social::SocialPost> = payload
        .items
        .into_iter()
        .filter_map(|item| {
            let video_id = item.id.video_id?;
            let thumb = item
                .snippet
                .thumbnails
                .high
                .or(item.snippet.thumbnails.medium)
                .or(item.snippet.thumbnails.default)
                .map(|t| t.url)
                .unwrap_or_default();
            Some(SocialPost {
                id: format!("yt-{}", video_id),
                platform: String::from("youtube"),
                title: item.snippet.title,
                url: format!("https://www.youtube.com/watch?v={}", video_id),
                thumbnail_url: Some(thumb),
                published_at: item.snippet.published_at,
            })
        })
        .collect();

    if posts.is_empty() {
        Ok(static_youtube_posts())
    } else {
        Ok(posts)
    }
}

fn static_youtube_posts() -> Vec<SocialPost> {
    let json = env::var("YEW_FULLSTACK_YOUTUBE_STATIC_JSON").unwrap_or_default();
    if json.is_empty() {
        return vec![];
    }
    serde_json::from_str(&json).unwrap_or_default()
}
