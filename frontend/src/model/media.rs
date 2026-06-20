use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaFeedItem {
    pub id: String,
    pub source: String,
    pub media_type: String,
    pub title: Option<String>,
    pub caption: Option<String>,
    pub thumbnail_url: String,
    pub media_url: Option<String>,
    pub link_url: Option<String>,
    pub published_at: Option<String>,
    pub collection_id: Option<String>,
    pub collection_kind: Option<String>,
    pub collection_title: Option<String>,
    pub featured: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaFeedResponse {
    pub items: Vec<MediaFeedItem>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InstagramFeatured {
    pub post_url: String,
    pub thumbnail_url: Option<String>,
    pub title: Option<String>,
    pub profile_url: String,
    pub handle: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrintSelection {
    pub id: String,
    pub item_id: String,
    pub source: String,
    pub title: Option<String>,
    pub thumbnail_url: String,
    pub media_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrintSelectionInput {
    pub item_id: String,
    pub source: String,
    pub title: Option<String>,
    pub thumbnail_url: String,
    pub media_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChangePassword {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EmailChange {
    pub new_email: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TotpVerify {
    pub code: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TotpEnrollResponse {
    pub secret: String,
    pub otpauth_url: String,
}
