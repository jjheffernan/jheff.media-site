use crate::{
    components::{
        ui::{Heading, HeadingLevel, Spinner},
        ContactForm,
    },
    utils::{format_event_local, timezone_caption},
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
    #[serde(default)]
    kind: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SchedulePayload {
    #[serde(default)]
    events: Vec<ScheduleEvent>,
}

fn event_list(events: &[ScheduleEvent]) -> Html {
    let events = &events[..events.len().min(8)];
    if events.is_empty() {
        return Html::default();
    }
    html! {
        <section class="space-y-3">
            <h2 class="title-sm">{ "Availability" }</h2>
            <p class="muted">{ timezone_caption() }</p>
            <ul class="space-y-3">
                { for events.iter().map(|event| {
                    let location = event.location.clone().filter(|value| !value.is_empty());
                    html! {
                        <li class="card">
                            <p>{ event.title.clone() }</p>
                            <p class="muted">{ event.kind.clone() }</p>
                            <p class="muted">{ format_event_local(&event.starts_at) }</p>
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
    // Don't block the booking form on /api/schedule — availability is optional chrome.
    let events = use_state(Vec::<ScheduleEvent>::new);
    let loading_avail = use_state(|| true);

    {
        let events = events.clone();
        let loading_avail = loading_avail.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match Request::get("/api/schedule").send().await {
                    Ok(resp) if resp.ok() => {
                        if let Ok(payload) = resp.json::<SchedulePayload>().await {
                            events.set(payload.events);
                        }
                    }
                    _ => {}
                }
                loading_avail.set(false);
            });
            || ()
        });
    }

    html! {
        <div class="space-y-6">
            <Heading
                level={HeadingLevel::H1}
                subtitle="Track days, media shoots, social content, and race engineering — request availability for @jheffmedia work."
            >
                { "Booking" }
            </Heading>
            if *loading_avail {
                <Spinner label="Loading availability…" />
            } else {
                { event_list(&events) }
            }
            <div class="card">
                <ContactForm booking=true />
            </div>
        </div>
    }
}
