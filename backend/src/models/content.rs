use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub id: String,
    pub title: Option<String>,
    pub thumbnail_url: String,
    pub media_url: Option<String>,
    pub media_type: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[serde(default)]
    pub featured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentCollection {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub kind: Option<String>,
    pub summary: Option<String>,
    pub cover_url: Option<String>,
    pub status: Option<String>,
    pub date: Option<String>,
    pub location: Option<String>,
    pub media: Vec<MediaItem>,
}

/// Partial update for gallery/shoot metadata (Mongo only).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateContentCollection {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub status: Option<String>,
    pub date: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderMediaRequest {
    pub media_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentListResponse {
    pub items: Vec<ContentCollectionSummary>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

fn default_contact_kind() -> String {
    "contact".into()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactSubmission {
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
    /// Honeypot. Humans leave this empty; filled values are dropped.
    #[serde(default)]
    pub website: String,
    #[serde(default = "default_contact_kind")]
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingConfig {
    pub embed_url: Option<String>,
    pub contact_email: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtherSite {
    pub name: String,
    pub url: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OtherSitesResponse {
    pub sites: Vec<OtherSite>,
    pub source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_item_featured_defaults_false() {
        let item: MediaItem =
            serde_json::from_str(r#"{"id":"a","thumbnailUrl":"/a"}"#).unwrap();
        assert!(!item.featured);
    }
}
