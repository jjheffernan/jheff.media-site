use crate::{
    components::ui::{Heading, HeadingLevel, Spinner, Text, TextTone},
    model::{BookingConfig, ServerResponse},
    routes::AppRoutes,
};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Ready(BookingConfig),
    Error(String),
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
                            state.set(LoadState::Ready(config));
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
        LoadState::Ready(config) if config.embed_url.is_none() => html! {
            <div class="space-y-4 rounded-2xl border border-border bg-surface-elevated p-6">
                <Text tone={TextTone::Muted}>
                    { "Book track instruction, media shoots, or race data engineering — send a message and we'll line up dates." }
                </Text>
                if let Some(email) = &config.contact_email {
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
        },
        LoadState::Ready(config) => html! {
            <div class="overflow-hidden rounded-2xl border border-border bg-surface-elevated">
                <iframe
                    class="min-h-[min(70vh,720px)] w-full border-0 bg-surface"
                    src={config.embed_url.clone().unwrap_or_default()}
                    title="Booking"
                    loading="lazy"
                />
            </div>
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
