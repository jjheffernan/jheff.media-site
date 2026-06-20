use crate::{
    components::ui::{Heading, HeadingLevel, Spinner, Text, TextTone},
    model::{OtherSite, OtherSitesResponse},
};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Ready(OtherSitesResponse),
    Error(String),
}

#[function_component(OtherSites)]
pub fn other_sites() -> Html {
    let state = use_state(|| LoadState::Loading);

    {
        let state = state.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let result = Request::get("/api/sites/other").send().await;
                match result {
                    Ok(resp) if resp.ok() => {
                        if let Ok(data) = resp.json::<OtherSitesResponse>().await {
                            state.set(LoadState::Ready(data));
                        } else {
                            state.set(LoadState::Error("Could not load other sites.".into()));
                        }
                    }
                    Ok(_) => state.set(LoadState::Error("Other sites request failed.".into())),
                    Err(err) => state.set(LoadState::Error(err.to_string())),
                }
            });
            || ()
        });
    }

    let content = match &*state {
        LoadState::Loading => html! { <Spinner label="Loading…" /> },
        LoadState::Error(msg) => html! {
            <p class="text-sm text-muted">{ format!("{}", msg) }</p>
        },
        LoadState::Ready(data) if data.sites.is_empty() => html! {
            <Text tone={TextTone::Muted}>
                { "No other sites configured." }
            </Text>
        },
        LoadState::Ready(data) => html! {
            <ul class="space-y-4">
                { for data.sites.iter().map(|site| html! {
                    <li
                        key={site.url.clone()}
                        class="rounded-2xl border border-border bg-surface-elevated p-5 transition hover:border-accent/50"
                    >
                        <a
                            class="text-lg font-semibold text-foreground hover:text-accent"
                            href={site.url.clone()}
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            { site.name.clone() }
                        </a>
                        if let Some(desc) = &site.description {
                            <p class="mt-2 text-sm text-muted">{ desc.clone() }</p>
                        }
                    </li>
                }) }
            </ul>
        },
    };

    html! {
        <div class="space-y-6">
            <Heading
                level={HeadingLevel::H1}
                subtitle="Other projects and brands — separate from this @jheffmedia hub."
            >
                { "Other sites" }
            </Heading>
            { content }
        </div>
    }
}
