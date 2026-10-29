use serde::{Deserialize, Serialize};

fn default_public() -> bool {
    true
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleEvent {
    pub id: String,
    pub title: String,
    pub starts_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleConfig {
    pub embed_url: Option<String>,
    pub source: String,
    pub events: Vec<ScheduleEvent>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateScheduleEvent {
    pub title: String,
    pub starts_at: String,
    #[serde(default)]
    pub location: Option<String>,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleEventRecord {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub title: String,
    pub starts_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    pub kind: String,
    #[serde(default = "default_public")]
    pub public: bool,
}
