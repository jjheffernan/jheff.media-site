use crate::{
    error::{ServiceError, ServiceResult},
    models::social::SocialPost,
    services::site_config_service::ResolvedConfig,
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

pub fn channel_url(cfg: &ResolvedConfig) -> String {
    if !cfg.youtube_channel_url.trim().is_empty() {
        cfg.youtube_channel_url.clone()
    } else {
        let handle = if cfg.youtube_handle.trim().is_empty() {
            DEFAULT_CHANNEL_HANDLE
        } else {
            cfg.youtube_handle.trim().trim_start_matches('@')
        };
        format!("https://www.youtube.com/@{}", handle)
    }
}

pub fn channel_handle(cfg: &ResolvedConfig) -> String {
    if !cfg.youtube_handle.trim().is_empty() {
        cfg.youtube_handle.clone()
    } else {
        format!("@{}", DEFAULT_CHANNEL_HANDLE)
    }
}

pub async fn fetch_channel_videos(
    client: &Client,
    limit: usize,
    cfg: &ResolvedConfig,
) -> ServiceResult<Vec<SocialPost>> {
    let api_key = cfg.youtube_api_key.trim();
    if api_key.is_empty() {
        return Ok(static_youtube_posts());
    }

    let channel_id = cfg.youtube_channel_id.trim();
    let url = if !channel_id.is_empty() {
        format!(
            "https://www.googleapis.com/youtube/v3/search?part=snippet&channelId={}&type=video&order=date&maxResults={}&key={}",
            channel_id,
            limit.clamp(1, 25),
            api_key
        )
    } else {
        let handle = cfg.youtube_handle.trim().trim_start_matches('@');
        let q = if handle.is_empty() {
            DEFAULT_CHANNEL_HANDLE
        } else {
            handle
        };
        format!(
            "https://www.googleapis.com/youtube/v3/search?part=snippet&q={}&type=channel&maxResults=1&key={}",
            q, api_key
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

    let posts: Vec<SocialPost> = payload
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
