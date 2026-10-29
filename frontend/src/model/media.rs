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
    #[serde(default)]
    pub collection_id: Option<String>,
    #[serde(default)]
    pub collection_kind: Option<String>,
    #[serde(default)]
    pub collection_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrintSelectionInput {
    pub item_id: String,
    pub source: String,
    pub title: Option<String>,
    pub thumbnail_url: String,
    pub media_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_title: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrintOrder {
    pub id: String,
    pub status: String,
    pub note: Option<String>,
    pub item_count: usize,
    pub created_at: String,
    pub size: String,
    pub unit_price_cents: i64,
    pub total_cents: i64,
    pub payment_status: String,
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
