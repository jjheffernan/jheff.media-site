use crate::{
    components::ui::{Heading, HeadingLevel, Spinner, Text, TextTone},
    model::{BookingConfig, ServerResponse},
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
struct SchedulePayload {
    #[serde(default)]
    events: Vec<ScheduleEvent>,
}

#[derive(Clone, PartialEq)]
struct BookingPage {
    config: BookingConfig,
    events: Vec<ScheduleEvent>,
}

#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Ready(BookingPage),
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
                    <li class="space-y-1 rounded-2xl border border-border bg-surface-elevated p-4">
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

async fn load_events() -> Vec<ScheduleEvent> {
    let Ok(resp) = Request::get("/api/schedule").send().await else {
        return vec![];
    };
    if !resp.ok() {
        return vec![];
    }
    resp.json::<SchedulePayload>()
        .await
        .map(|payload| payload.events)
        .unwrap_or_default()
}

#[function_component(Booking)]
pub fn booking() -> Html {
    let state = use_state(|| LoadState::Loading);

    {
        let state = state.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let result = Request::get("/api/booking").send().await;
                match result {
                    Ok(resp) if resp.ok() => {
                        if let Ok(config) = resp.json::<BookingConfig>().await {
                            let events = load_events().await;
                            state.set(LoadState::Ready(BookingPage { config, events }));
                            return;
                        }
                        state.set(LoadState::Error("Could not parse booking config.".into()));
                    }
                    Ok(resp) => {
                        if let Ok(body) = resp.json::<ServerResponse<String>>().await {
                            state.set(LoadState::Error(body.message));
                        } else {
                            state.set(LoadState::Error("Booking request failed.".into()));
                        }
                    }
                    Err(err) => state.set(LoadState::Error(err.to_string())),
                }
            });
            || ()
        });
    }

    let content = match &*state {
        LoadState::Loading => html! { <Spinner label="Loading booking…" /> },
        LoadState::Error(msg) => html! {
            <p class="text-sm text-muted">{ format!("Booking unavailable: {}", msg) }</p>
        },
        LoadState::Ready(page) => {
            let panel = if page.config.embed_url.is_none() {
                html! {
                    <div class="space-y-4 rounded-2xl border border-border bg-surface-elevated p-6">
                        <Text tone={TextTone::Muted}>
                            { "Book track instruction, media shoots, or race data engineering — send a message and we'll line up dates." }
                        </Text>
                        if let Some(email) = &page.config.contact_email {
                            <p class="text-sm text-foreground">
                                { "Email: " }
                                <a class="text-accent hover:underline" href={format!("mailto:{}", email)}>
                                    { email.clone() }
                                </a>
                            </p>
                        }
                        <Link<AppRoutes>
                            to={AppRoutes::Contact}
                            classes="inline-flex rounded-lg bg-accent px-4 py-2 text-sm font-medium text-zinc-950 transition hover:bg-accent-hover"
                        >
                            { "Contact & book" }
                        </Link<AppRoutes>>
                    </div>
                }
            } else {
                html! {
                    <div class="overflow-hidden rounded-2xl border border-border bg-surface-elevated">
                        <iframe
                            class="min-h-[min(70vh,720px)] w-full border-0 bg-surface"
                            src={page.config.embed_url.clone().unwrap_or_default()}
                            title="Booking"
                            loading="lazy"
                        />
                    </div>
                }
            };
            html! {
                <>
                    { event_list(&page.events) }
                    { panel }
                </>
            }
        }
    };

    html! {
        <div class="space-y-6">
            <Heading
                level={HeadingLevel::H1}
                subtitle="Track days, media shoots, social content, and race engineering — request availability for @jheffmedia work."
            >
                { "Booking" }
            </Heading>
            { content }
        </div>
    }
}
