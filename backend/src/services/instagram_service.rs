use crate::{
    error::{ServiceError, ServiceResult},
    models::social::SocialPost,
};
use actix_web::http::StatusCode;
use awc::Client;
use serde::Deserialize;
use std::env;

const DEFAULT_LIMIT: usize = 12;
const INSTAGRAM_HANDLE: &str = "jheffmedia";
const INSTAGRAM_PROFILE_URL: &str = "https://www.instagram.com/jheffmedia/";

#[derive(Debug, Deserialize)]
struct InstagramMediaList {
    data: Vec<InstagramMedia>,
}

#[derive(Debug, Deserialize)]
struct InstagramMedia {
    id: String,
    caption: Option<String>,
    media_type: Option<String>,
    media_url: Option<String>,
    thumbnail_url: Option<String>,
    permalink: Option<String>,
    timestamp: Option<String>,
}

pub fn instagram_profile_url() -> String {
    env::var("YEW_FULLSTACK_INSTAGRAM_PROFILE_URL")
        .ok()
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| INSTAGRAM_PROFILE_URL.to_string())
}

pub fn instagram_handle() -> String {
    env::var("YEW_FULLSTACK_INSTAGRAM_HANDLE")
        .ok()
        .filter(|h| !h.is_empty())
        .unwrap_or_else(|| format!("@{}", INSTAGRAM_HANDLE))
}

pub async fn fetch_instagram_posts(client: &Client, limit: usize) -> ServiceResult<Vec<SocialPost>> {
    let token = env::var("YEW_FULLSTACK_INSTAGRAM_ACCESS_TOKEN")
        .ok()
        .filter(|t| !t.is_empty());

    if token.is_none() {
        return Ok(vec![]);
    }

    let token = token.unwrap();
    let limit = limit.clamp(1, 25);
    let url = format!(
        "https://graph.instagram.com/me/media?fields=id,caption,media_type,media_url,thumbnail_url,permalink,timestamp&limit={}&access_token={}",
        limit, token
    );

    let mut response = client
        .get(&url)
        .send()
        .await
        .map_err(|err| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.body().await.unwrap_or_default();
        let message = String::from_utf8_lossy(&body).to_string();
        error!("Instagram API error {}: {}", status, message);
        return Err(ServiceError::new(
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY),
            format!("Instagram API request failed: {}", status),
        ));
    }

    let payload: InstagramMediaList = response
        .json()
        .await
        .map_err(|err| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(payload
        .data
        .into_iter()
        .map(|item| {
            let title = item
                .caption
                .as_ref()
                .map(|c| c.lines().next().unwrap_or(c).trim())
                .filter(|s| !s.is_empty())
                .map(|s| {
                    if s.len() > 120 {
                        format!("{}…", s.chars().take(120).collect::<String>())
                    } else {
                        s.to_string()
                    }
                })
                .unwrap_or_else(|| "Instagram post".to_string());

            let thumbnail_url = item.thumbnail_url.or(item.media_url.clone());

            SocialPost {
                id: item.id,
                platform: String::from("instagram"),
                title,
                url: item
                    .permalink
                    .unwrap_or_else(|| instagram_profile_url()),
                thumbnail_url,
                published_at: item.timestamp,
            }
        })
        .collect())
}

pub async fn instagram_posts_or_fallback(
    client: &Client,
    limit: usize,
) -> (Vec<SocialPost>, String) {
    match fetch_instagram_posts(client, limit).await {
        Ok(posts) if !posts.is_empty() => (posts, String::from("instagram")),
        Ok(_) => (vec![], String::from("none")),
        Err(err) => {
            warn!("Instagram fetch failed: {}", err.body.message);
            (vec![], String::from("error"))
        }
    }
}
