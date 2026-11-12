use crate::{
    models::social::{SocialAccount, SocialHubResponse},
    services::{instagram_service, site_config_service, youtube_service},
};
use awc::Client;
use mongodb::Database;
use std::env;

pub async fn hub_config(db: &Database, client: &Client) -> SocialHubResponse {
    let cfg = site_config_service::resolved(db).await;
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

    let instagram_handle = instagram_service::instagram_handle(&cfg);
    let instagram_profile = instagram_service::instagram_profile_url(&cfg);

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
            handle: youtube_service::channel_handle(&cfg),
            profile_url: youtube_service::channel_url(&cfg),
            display_name: Some(String::from("jheffmedia")),
        });
    }

    // Admin "Other" integration sits above env extras.
    if let Some(other) = other_from_cfg(&cfg) {
        if !accounts.iter().any(|a| a.platform == other.platform) {
            accounts.push(SocialAccount {
                platform: other.platform.clone(),
                handle: other.handle.clone(),
                profile_url: other.profile_url.clone(),
                display_name: None,
            });
        }
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
        instagram_service::instagram_posts_or_fallback(client, 12, &cfg).await;
    let youtube_posts = youtube_service::fetch_channel_videos(client, 6, &cfg)
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

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SocialAdminResponse {
    pub config: site_config_service::SocialConfigView,
    pub channels: Vec<ChannelView>,
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

struct OtherChannel {
    platform: String,
    handle: String,
    profile_url: String,
}

fn other_from_cfg(cfg: &site_config_service::ResolvedConfig) -> Option<OtherChannel> {
    let platform = cfg.other_platform.trim().to_ascii_lowercase();
    let handle = cfg.other_handle.trim();
    let profile_url = cfg.other_profile_url.trim();
    if platform.is_empty() || handle.is_empty() || profile_url.is_empty() {
        return None;
    }
    Some(OtherChannel {
        platform,
        handle: handle.to_string(),
        profile_url: profile_url.to_string(),
    })
}

pub async fn admin_social(db: &Database, client: &Client) -> SocialAdminResponse {
    let cfg_view = site_config_service::get_social_view(db).await;
    let channels = channels(db, client).await;
    SocialAdminResponse {
        config: cfg_view,
        channels,
    }
}

pub async fn channels(db: &Database, client: &Client) -> Vec<ChannelView> {
    let cfg = site_config_service::resolved(db).await;
    let hub = hub_config(db, client).await;
    hub.accounts
        .into_iter()
        .map(|account| {
            let key_env = key_env_for(&account.platform);
            let keyed = match account.platform.as_str() {
                "instagram" => !cfg.instagram_access_token.trim().is_empty(),
                "youtube" => !cfg.youtube_api_key.trim().is_empty(),
                other if other == cfg.other_platform => !cfg.other_api_key.trim().is_empty(),
                _ => env_ready(&key_env),
            };
            let posts = hub
                .posts
                .iter()
                .filter(|post| post.platform == account.platform)
                .take(4)
                .cloned()
                .collect();
            ChannelView {
                keyed,
                platform: account.platform,
                handle: account.handle,
                profile_url: account.profile_url,
                key_env,
                posts,
            }
        })
        .collect()
}

/// Public `/links` integrations — profile URLs when connected (token/key or profile set).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicIntegrationLink {
    pub platform: String,
    pub label: String,
    pub url: String,
    pub live: bool,
}

pub async fn public_integration_links(db: &Database) -> Vec<PublicIntegrationLink> {
    let cfg = site_config_service::resolved(db).await;
    let mut out = Vec::new();

    let ig_url = instagram_service::instagram_profile_url(&cfg);
    let ig_live = !cfg.instagram_access_token.trim().is_empty();
    if ig_live || !cfg.instagram_profile_url.trim().is_empty() {
        out.push(PublicIntegrationLink {
            platform: "instagram".into(),
            label: "Instagram".into(),
            url: ig_url,
            live: ig_live,
        });
    }

    let yt_url = youtube_service::channel_url(&cfg);
    let yt_live = !cfg.youtube_api_key.trim().is_empty();
    if yt_live
        || !cfg.youtube_channel_url.trim().is_empty()
        || !cfg.youtube_channel_id.trim().is_empty()
    {
        out.push(PublicIntegrationLink {
            platform: "youtube".into(),
            label: "YouTube".into(),
            url: yt_url,
            live: yt_live,
        });
    }

    if let Some(other) = other_from_cfg(&cfg) {
        let label = {
            let mut chars = other.platform.chars();
            match chars.next() {
                Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str()),
                None => other.platform.clone(),
            }
        };
        out.push(PublicIntegrationLink {
            platform: other.platform,
            label,
            url: other.profile_url,
            live: !cfg.other_api_key.trim().is_empty(),
        });
    }

    out
}

fn merge_posts(
    static_posts: Vec<crate::models::social::SocialPost>,
    instagram: Vec<crate::models::social::SocialPost>,
    youtube: Vec<crate::models::social::SocialPost>,
) -> Vec<crate::models::social::SocialPost> {
    let mut posts = Vec::new();
    if instagram.is_empty() {
        posts.extend(
            static_posts
                .iter()
                .filter(|post| post.platform == "instagram")
                .cloned(),
        );
    } else {
        posts.extend(instagram);
    }
    if youtube.is_empty() {
        posts.extend(
            static_posts
                .iter()
                .filter(|post| post.platform == "youtube")
                .cloned(),
        );
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
