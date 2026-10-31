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

    for channel in extra_channels() {
        if accounts.iter().any(|account| account.platform == channel.platform) {
            continue;
        }
        accounts.push(SocialAccount {
            platform: channel.platform,
            handle: channel.handle,
            profile_url: channel.profile_url,
            display_name: None,
        });
    }

    let (instagram_posts, instagram_source) =
        instagram_service::instagram_posts_or_fallback(client, 12).await;
    let youtube_posts = youtube_service::fetch_channel_videos(client, 6)
        .await
        .unwrap_or_default();
    let has_instagram = !instagram_posts.is_empty();
    let posts = merge_posts(static_posts, instagram_posts, youtube_posts);

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

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelView {
    pub platform: String,
    pub handle: String,
    pub profile_url: String,
    pub key_env: String,
    pub keyed: bool,
    pub posts: Vec<crate::models::social::SocialPost>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChannelConfig {
    platform: String,
    handle: String,
    profile_url: String,
    #[serde(default)]
    key_env: String,
}

pub async fn channels(client: &Client) -> Vec<ChannelView> {
    let hub = hub_config(client).await;
    hub.accounts
        .into_iter()
        .map(|account| {
            let key_env = key_env_for(&account.platform);
            let posts = hub
                .posts
                .iter()
                .filter(|post| post.platform == account.platform)
                .take(4)
                .cloned()
                .collect();
            ChannelView {
                keyed: env_ready(&key_env),
                platform: account.platform,
                handle: account.handle,
                profile_url: account.profile_url,
                key_env,
                posts,
            }
        })
        .collect()
}

fn merge_posts(
    static_posts: Vec<crate::models::social::SocialPost>,
    instagram: Vec<crate::models::social::SocialPost>,
    youtube: Vec<crate::models::social::SocialPost>,
) -> Vec<crate::models::social::SocialPost> {
    let mut posts = Vec::new();
    if instagram.is_empty() {
        posts.extend(static_posts.iter().filter(|post| post.platform == "instagram").cloned());
    } else {
        posts.extend(instagram);
    }
    if youtube.is_empty() {
        posts.extend(static_posts.iter().filter(|post| post.platform == "youtube").cloned());
    } else {
        posts.extend(youtube);
    }
    posts.extend(
        static_posts
            .into_iter()
            .filter(|post| post.platform != "instagram" && post.platform != "youtube"),
    );
    posts
}

fn key_env_for(platform: &str) -> String {
    match platform {
        "instagram" => "YEW_FULLSTACK_INSTAGRAM_ACCESS_TOKEN".into(),
        "youtube" => "YEW_FULLSTACK_YOUTUBE_API_KEY".into(),
        _ => extra_channels()
            .into_iter()
            .find(|channel| channel.platform == platform)
            .map(|channel| channel.key_env)
            .unwrap_or_default(),
    }
}

fn env_ready(name: &str) -> bool {
    !name.is_empty()
        && env::var(name)
            .ok()
            .is_some_and(|value| !value.trim().is_empty())
}

fn extra_channels() -> Vec<ChannelConfig> {
    let json = env::var("YEW_FULLSTACK_SOCIAL_CHANNELS").unwrap_or_default();
    if json.trim().is_empty() {
        return Vec::new();
    }
    let parsed: Vec<ChannelConfig> = match serde_json::from_str(&json) {
        Ok(rows) => rows,
        Err(err) => {
            error!("Invalid YEW_FULLSTACK_SOCIAL_CHANNELS: {err}");
            return Vec::new();
        }
    };
    parsed.into_iter().filter_map(clean_channel).take(8).collect()
}

fn clean_channel(raw: ChannelConfig) -> Option<ChannelConfig> {
    let platform = raw.platform.trim().to_ascii_lowercase();
    let handle = raw.handle.trim();
    let profile_url = raw.profile_url.trim();
    let key_env = raw.key_env.trim();
    let platform_ok = (2..=20).contains(&platform.len())
        && platform
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
        && platform != "instagram"
        && platform != "youtube";
    let url_ok = (profile_url.starts_with("https://") || profile_url.starts_with("http://"))
        && profile_url.len() <= 200
        && !profile_url.chars().any(char::is_whitespace);
    if !platform_ok || handle.is_empty() || handle.len() > 40 || !url_ok {
        return None;
    }
    if !key_env.is_empty() && !valid_key_env(key_env) {
        return None;
    }
    Some(ChannelConfig {
        platform,
        handle: handle.to_string(),
        profile_url: profile_url.to_string(),
        key_env: key_env.to_string(),
    })
}

fn valid_key_env(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("YEW_FULLSTACK_") else {
        return false;
    };
    (1..=40).contains(&rest.len())
        && rest
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

#[cfg(test)]
mod tests {
    use super::{clean_channel, merge_posts, valid_key_env, ChannelConfig};
    use crate::models::social::SocialPost;

    fn post(platform: &str) -> SocialPost {
        SocialPost {
            id: platform.into(),
            platform: platform.into(),
            title: platform.into(),
            url: format!("https://example.com/{platform}"),
            thumbnail_url: None,
            published_at: None,
        }
    }

    #[test]
    fn other_platform_posts_stay_when_the_live_feeds_are_empty() {
        let posts = merge_posts(vec![post("instagram"), post("tiktok")], vec![], vec![]);
        let names: Vec<_> = posts.iter().map(|post| post.platform.as_str()).collect();
        assert_eq!(names, ["instagram", "tiktok"]);
    }

    #[test]
    fn live_instagram_replaces_only_instagram_static_posts() {
        let posts = merge_posts(
            vec![post("instagram"), post("tiktok")],
            vec![post("instagram")],
            vec![],
        );
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0].id, "instagram");
        assert_eq!(posts[1].platform, "tiktok");
    }

    #[test]
    fn key_env_stays_inside_the_site_prefix() {
        assert!(valid_key_env("YEW_FULLSTACK_TIKTOK_KEY"));
        assert!(!valid_key_env("PATH"));
        assert!(!valid_key_env("YEW_FULLSTACK_"));
        assert!(!valid_key_env("yew_fullstack_tiktok"));
    }

    #[test]
    fn a_bad_channel_is_dropped() {
        let ok = clean_channel(ChannelConfig {
            platform: "TikTok".into(),
            handle: "@jheffmedia".into(),
            profile_url: "https://www.tiktok.com/@jheffmedia".into(),
            key_env: "YEW_FULLSTACK_TIKTOK_KEY".into(),
        });
        assert_eq!(ok.unwrap().platform, "tiktok");
        assert!(clean_channel(ChannelConfig {
            platform: "tiktok".into(),
            handle: "@jheffmedia".into(),
            profile_url: "https://www.tiktok.com/@jheffmedia".into(),
            key_env: "PATH".into(),
        })
        .is_none());
        assert!(clean_channel(ChannelConfig {
            platform: "instagram".into(),
            handle: "@jheffmedia".into(),
            profile_url: "https://instagram.com/jheffmedia".into(),
            key_env: String::new(),
        })
        .is_none());
    }
}
