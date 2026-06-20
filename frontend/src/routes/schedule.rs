use crate::{
    components::ui::{Heading, HeadingLevel, Spinner, Text, TextTone},
    model::ServerResponse,
};
use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ScheduleConfig {
    embed_url: Option<String>,
    source: String,
}

#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Ready(ScheduleConfig),
    Error(String),
}

#[function_component(Schedule)]
pub fn schedule() -> Html {
    let state = use_state(|| LoadState::Loading);

    {
        let state = state.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let result = Request::get("/api/schedule").send().await;
                match result {
                    Ok(resp) if resp.ok() => {
                        if let Ok(config) = resp.json::<ScheduleConfig>().await {
                            state.set(LoadState::Ready(config));
                            return;
                        }
                        state.set(LoadState::Error("Could not parse schedule config.".into()));
                    }
                    Ok(resp) => {
                        if let Ok(body) = resp.json::<ServerResponse<String>>().await {
                            state.set(LoadState::Error(body.message));
                        } else {
                            state.set(LoadState::Error("Schedule request failed.".into()));
                        }
                    }
                    Err(err) => state.set(LoadState::Error(err.to_string())),
                }
            });
            || ()
        });
    }

    let content = match &*state {
        LoadState::Loading => html! { <Spinner label="Loading schedule…" /> },
        LoadState::Error(msg) => html! {
            <p class="text-sm text-muted">{ format!("Schedule unavailable: {}", msg) }</p>
        },
        LoadState::Ready(config) if config.embed_url.is_none() => html! {
            <Text tone={TextTone::Muted}>
                { "No schedule embed configured. Set YEW_FULLSTACK_SCHEDULE_EMBED_URL on the backend." }
            </Text>
        },
        LoadState::Ready(config) => html! {
            <div class="overflow-hidden rounded-2xl border border-border bg-surface-elevated">
                <iframe
                    class="min-h-[min(70vh,720px)] w-full border-0 bg-surface"
                    src={config.embed_url.clone().unwrap_or_default()}
                    title="Shoot schedule"
                    loading="lazy"
                />
            </div>
        },
    };

    html! {
        <div class="space-y-6">
            <Heading
                level={HeadingLevel::H1}
                subtitle="Shoot locations and dates load from your external scheduling service — nothing is hardcoded here."
            >
                { "Schedule" }
            </Heading>
            { content }
        </div>
    }
}
