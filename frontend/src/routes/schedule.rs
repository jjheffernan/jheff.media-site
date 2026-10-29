use crate::{
    components::ui::{Heading, HeadingLevel, Spinner, Text, TextTone},
    model::ServerResponse,
    routes::AppRoutes,
};
use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ScheduleEvent {
    title: String,
    starts_at: String,
    #[serde(default)]
    location: Option<String>,
    kind: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ScheduleConfig {
    embed_url: Option<String>,
    source: String,
    #[serde(default)]
    events: Vec<ScheduleEvent>,
}

#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Ready(ScheduleConfig),
    Error(String),
}

fn event_list(events: &[ScheduleEvent]) -> Html {
    if events.is_empty() {
        return Html::default();
    }
    html! {
        <ul class="space-y-3">
            { for events.iter().map(|event| {
                let location = event.location.clone().filter(|value| !value.is_empty());
                html! {
                    <li class="card space-y-1">
                        <p class="font-medium text-foreground">{ event.title.clone() }</p>
                        <p class="text-sm text-muted">{ event.kind.clone() }</p>
                        <p class="text-sm text-muted">{ event.starts_at.clone() }</p>
                        if let Some(location) = location {
                            <p class="text-sm text-muted">{ location }</p>
                        }
                    </li>
                }
            }) }
        </ul>
    }
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
        LoadState::Ready(config) => {
            let panel = if config.embed_url.is_none() {
                html! {
                    <div class="card space-y-4">
                        <Text tone={TextTone::Muted}>
                            { "Shoot dates aren't published here yet. Send a booking note and we'll line up the calendar." }
                        </Text>
                        <Link<AppRoutes>
                            to={AppRoutes::Contact}
                            classes="inline-flex btn btn-primary"
                        >
                            { "Contact & book" }
                        </Link<AppRoutes>>
                    </div>
                }
            } else {
                html! {
                    <div class="card card-flat overflow-hidden">
                        <iframe
                            class="min-h-[min(70vh,720px)] w-full border-0 bg-surface"
                            src={config.embed_url.clone().unwrap_or_default()}
                            title="Shoot schedule"
                            loading="lazy"
                        />
                    </div>
                }
            };
            html! {
                <>
                    { event_list(&config.events) }
                    { panel }
                </>
            }
        }
    };

    html! {
        <div class="space-y-6">
            <Heading
                level={HeadingLevel::H1}
                subtitle="Published shoot dates, or a note to book if the calendar isn't live yet."
            >
                { "Schedule" }
            </Heading>
            { content }
        </div>
    }
}
