use crate::models::schedule::ScheduleConfig;
use std::env;

pub fn schedule_config() -> ScheduleConfig {
    let embed_url = env::var("YEW_FULLSTACK_SCHEDULE_EMBED_URL")
        .ok()
        .filter(|url| !url.is_empty());
    let source = if embed_url.is_some() {
        String::from("embed")
    } else {
        String::from("none")
    };

    ScheduleConfig { embed_url, source }
}
