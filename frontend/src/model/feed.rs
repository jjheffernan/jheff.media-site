use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeedItem {
    pub id: String,
    pub title: Option<String>,
    pub thumbnail_url: String,
    pub media_url: Option<String>,
    pub media_type: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeedResponse {
    pub items: Vec<FeedItem>,
    pub source: String,
}
