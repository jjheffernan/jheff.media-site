use crate::{
    components::ui::{Heading, HeadingLevel, Spinner, Text, TextTone},
    model::{OtherSite, OtherSitesResponse},
    services::get_json,
};
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
                match get_json::<OtherSitesResponse>("/api/sites/other").await {
                    Ok(data) => state.set(LoadState::Ready(data)),
                    Err(err) => state.set(LoadState::Error(err)),
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
                        class="card"
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
