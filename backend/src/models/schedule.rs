use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleConfig {
    pub embed_url: Option<String>,
    pub source: String,
}
