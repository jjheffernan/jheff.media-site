use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SocialAccount {
    pub platform: String,
    pub handle: String,
    pub profile_url: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SocialPost {
    pub id: String,
    pub platform: String,
    pub title: String,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SocialHubResponse {
    pub accounts: Vec<SocialAccount>,
    pub posts: Vec<SocialPost>,
    pub source: String,
}
