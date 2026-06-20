use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
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
pub struct ContentCollection {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub cover_url: Option<String>,
    pub status: Option<String>,
    pub date: Option<String>,
    pub location: Option<String>,
    pub media: Vec<MediaItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContentCollectionSummary {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub cover_url: Option<String>,
    pub status: Option<String>,
    pub date: Option<String>,
    pub location: Option<String>,
    pub media_count: usize,
    pub has_video: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContentListResponse {
    pub items: Vec<ContentCollectionSummary>,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BookingConfig {
    pub embed_url: Option<String>,
    pub contact_email: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContactSubmission {
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OtherSite {
    pub name: String,
    pub url: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OtherSitesResponse {
    pub sites: Vec<OtherSite>,
    pub source: String,
}
