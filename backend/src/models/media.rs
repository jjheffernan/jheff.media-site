use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaFeedResponse {
    pub items: Vec<MediaFeedItem>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstagramFeatured {
    pub post_url: String,
    pub thumbnail_url: Option<String>,
    pub title: Option<String>,
    pub profile_url: String,
    pub handle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintSelection {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub user_id: bson::oid::ObjectId,
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
    pub created_at: bson::DateTime,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintSelectionView {
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

impl From<PrintSelection> for PrintSelectionView {
    fn from(selection: PrintSelection) -> Self {
        PrintSelectionView {
            id: selection.id.map(|oid| oid.to_hex()).unwrap_or_default(),
            item_id: selection.item_id,
            source: selection.source,
            title: selection.title,
            thumbnail_url: selection.thumbnail_url,
            media_url: selection.media_url,
            collection_id: selection.collection_id,
            collection_kind: selection.collection_kind,
            collection_title: selection.collection_title,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintSelectionInput {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintOrderItem {
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

fn default_unpaid() -> String {
    "unpaid".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintOrder {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub user_id: bson::oid::ObjectId,
    pub status: String,
    pub note: Option<String>,
    #[serde(default)]
    pub size: String,
    #[serde(default)]
    pub unit_price_cents: i64,
    #[serde(default)]
    pub total_cents: i64,
    #[serde(default = "default_unpaid")]
    pub payment_status: String,
    pub items: Vec<PrintOrderItem>,
    pub created_at: bson::DateTime,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintOrderView {
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintCheckoutInput {
    pub size: String,
    #[serde(default)]
    pub note: Option<String>,
}
