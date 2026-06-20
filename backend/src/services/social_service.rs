use crate::{
    models::social::{SocialAccount, SocialHubResponse},
    services::{instagram_service, youtube_service},
};
use awc::Client;
use std::env;

pub async fn hub_config(client: &Client) -> SocialHubResponse {
    let json = env::var("YEW_FULLSTACK_SOCIAL_HUB_JSON").unwrap_or_default();
    let mut accounts: Vec<SocialAccount> = vec![];
    let mut static_posts: Vec<crate::models::social::SocialPost> = vec![];

    if !json.is_empty() {
        if let Ok(config) = serde_json::from_str::<crate::models::social::SocialHubConfig>(&json) {
            accounts = config.accounts;
            static_posts = config.posts;
        } else {
            error!("Invalid YEW_FULLSTACK_SOCIAL_HUB_JSON");
        }
    }

    let instagram_handle = instagram_service::instagram_handle();
    let instagram_profile = instagram_service::instagram_profile_url();

    if !accounts.iter().any(|a| a.platform == "instagram") {
        accounts.insert(
            0,
            SocialAccount {
                platform: String::from("instagram"),
                handle: instagram_handle.clone(),
                profile_url: instagram_profile.clone(),
                display_name: Some(String::from("jheffmedia")),
            },
        );
    }

    if !accounts.iter().any(|a| a.platform == "youtube") {
        accounts.push(SocialAccount {
            platform: String::from("youtube"),
            handle: String::from("@jheffmedia"),
            profile_url: youtube_service::channel_url(),
            display_name: Some(String::from("jheffmedia")),
        });
    }

    let (instagram_posts, instagram_source) =
        instagram_service::instagram_posts_or_fallback(client, 12).await;

    let has_instagram = !instagram_posts.is_empty();
    let mut posts = if has_instagram {
        instagram_posts
    } else {
        static_posts
            .into_iter()
            .filter(|p| p.platform == "instagram")
            .collect()
    };

    if let Ok(youtube_posts) = youtube_service::fetch_channel_videos(client, 6).await {
        posts.extend(youtube_posts);
    }

    let source = if has_instagram {
        instagram_source
    } else if posts.is_empty() {
        String::from("none")
    } else {
        String::from("config")
    };

    SocialHubResponse {
        accounts,
        posts,
        source,
    }
}
