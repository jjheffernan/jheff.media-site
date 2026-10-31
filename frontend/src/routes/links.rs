use crate::components::ui::{Heading, HeadingLevel, Spinner, Text, TextTone};
use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq, Deserialize)]
struct SocialLink {
    label: String,
    url: String,
}

#[derive(Deserialize)]
struct LinksResponse {
    links: Vec<SocialLink>,
}

#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Ready(Vec<SocialLink>),
    Error(String),
}

#[function_component(Links)]
pub fn links() -> Html {
    let state = use_state(|| LoadState::Loading);

    {
        let state = state.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match Request::get("/api/links").send().await {
                    Ok(resp) if resp.ok() => match resp.json::<LinksResponse>().await {
                        Ok(body) => state.set(LoadState::Ready(body.links)),
                        Err(_) => state.set(LoadState::Error("Could not read links.".into())),
                    },
                    Ok(_) => state.set(LoadState::Error("Links request failed.".into())),
                    Err(err) => state.set(LoadState::Error(err.to_string())),
                }
            });
            || ()
        });
    }

    let body = match &*state {
        LoadState::Loading => html! { <Spinner label="Loading…" /> },
        LoadState::Error(msg) => html! { <p class="muted">{ msg.clone() }</p> },
        LoadState::Ready(links) if links.is_empty() => html! {
            <Text tone={TextTone::Muted}>{ "No social links yet." }</Text>
        },
        LoadState::Ready(links) => html! {
            <div class="link-stack">
                { for links.iter().enumerate().map(|(i, link)| html! {
                    <a
                        key={format!("{i}-{}", link.url)}
                        class="btn btn-quiet"
                        href={link.url.clone()}
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        { link.label.clone() }
                    </a>
                }) }
            </div>
        },
    };

    html! {
        <div class="link-page">
            <Heading level={HeadingLevel::H1} subtitle="@jheffmedia">
                { "Links" }
            </Heading>
            { body }
        </div>
    }
}
