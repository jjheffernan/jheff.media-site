use crate::{
    components::{
        ui::{Heading, HeadingLevel, Spinner},
        ContactForm,
    },
    model::ServerResponse,
};
use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

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
enum LoadState {
    Loading,
    Ready(Vec<ScheduleEvent>),
    Error(String),
}

fn event_list(events: &[ScheduleEvent]) -> Html {
    let events = &events[..events.len().min(8)];
    if events.is_empty() {
        return Html::default();
    }
    html! {
        <section class="space-y-3">
            <h2 class="title-sm">{ "Availability" }</h2>
            <ul class="space-y-3">
                { for events.iter().map(|event| {
                    let location = event.location.clone().filter(|value| !value.is_empty());
                    html! {
                        <li class="card">
                            <p>{ event.title.clone() }</p>
                            <p class="muted">{ event.kind.clone() }</p>
                            <p class="muted">{ event.starts_at.clone() }</p>
                            if let Some(location) = location {
                                <p class="muted">{ location }</p>
                            }
                        </li>
                    }
                }) }
            </ul>
        </section>
    }
}

#[function_component(Booking)]
pub fn booking() -> Html {
    let state = use_state(|| LoadState::Loading);

    {
        let state = state.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let result = Request::get("/api/schedule").send().await;
                match result {
                    Ok(resp) if resp.ok() => {
                        if let Ok(payload) = resp.json::<SchedulePayload>().await {
                            state.set(LoadState::Ready(payload.events));
                            return;
                        }
                        state.set(LoadState::Error("Could not parse schedule.".into()));
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
        LoadState::Loading => html! { <Spinner label="Loading booking…" /> },
        LoadState::Error(msg) => html! {
            <>
                <p class="muted">{ format!("Availability unavailable: {}", msg) }</p>
                <div class="card">
                    <ContactForm booking=true />
                </div>
            </>
        },
        LoadState::Ready(events) => html! {
            <>
                { event_list(events) }
                <div class="card">
                    <ContactForm booking=true />
                </div>
            </>
        },
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
