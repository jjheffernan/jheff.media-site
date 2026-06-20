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
}

impl From<PrintSelection> for PrintSelectionView {
    fn from(selection: PrintSelection) -> Self {
        PrintSelectionView {
            id: selection
                .id
                .map(|oid| oid.to_hex())
                .unwrap_or_default(),
            item_id: selection.item_id,
            source: selection.source,
            title: selection.title,
            thumbnail_url: selection.thumbnail_url,
            media_url: selection.media_url,
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
}
