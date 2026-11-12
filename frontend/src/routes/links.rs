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

#[derive(Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IntegrationLink {
    platform: String,
    label: String,
    url: String,
    live: bool,
}

#[derive(Deserialize)]
struct LinksResponse {
    links: Vec<SocialLink>,
    #[serde(default)]
    integrations: Vec<IntegrationLink>,
}

#[derive(Clone, PartialEq)]
struct PageData {
    integrations: Vec<IntegrationLink>,
    links: Vec<SocialLink>,
}

#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Ready(PageData),
    Error(String),
}

fn slug_label(label: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    let s = out.trim_matches('-').to_string();
    if s.is_empty() {
        "link".into()
    } else {
        s
    }
}

fn track_link_click(label: &str) {
    let slug = slug_label(label);
    // Fits telemetry clean_path: /links/out/<slug>
    let path = format!("/links/out/{slug}");
    let body = format!(r#"{{"path":"{}"}}"#, path.replace('\\', "").replace('"', ""));
    spawn_local(async move {
        if let Ok(req) = Request::post("/api/telemetry")
            .header("Content-Type", "application/json")
            .body(body)
        {
            let _ = req.send().await;
        }
    });
}

fn urls_match(a: &str, b: &str) -> bool {
    a.trim().trim_end_matches('/').eq_ignore_ascii_case(b.trim().trim_end_matches('/'))
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
                        Ok(body) => state.set(LoadState::Ready(PageData {
                            integrations: body.integrations,
                            links: body.links,
                        })),
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
        LoadState::Ready(page)
            if page.integrations.is_empty() && page.links.is_empty() =>
        {
            html! { <Text tone={TextTone::Muted}>{ "No social links yet." }</Text> }
        }
        LoadState::Ready(page) => {
            let integration_urls: Vec<_> = page.integrations.iter().map(|i| i.url.clone()).collect();
            let static_links: Vec<_> = page
                .links
                .iter()
                .filter(|link| {
                    !integration_urls
                        .iter()
                        .any(|iu| urls_match(iu, &link.url))
                })
                .cloned()
                .collect();
            html! {
                <div class="link-stack">
                    { for page.integrations.iter().map(|item| {
                        let label = item.label.clone();
                        let url = item.url.clone();
                        let live = item.live;
                        html! {
                            <a
                                key={format!("int-{}", item.platform)}
                                class="btn btn-quiet"
                                href={url}
                                target="_blank"
                                rel="noopener noreferrer"
                                onclick={Callback::from(move |_| track_link_click(&label))}
                            >
                                { item.label.clone() }
                                if live {
                                    <span class="muted">{ " · live" }</span>
                                }
                            </a>
                        }
                    }) }
                    { for static_links.iter().enumerate().map(|(i, link)| {
                        let label = link.label.clone();
                        let url = link.url.clone();
                        html! {
                            <a
                                key={format!("{i}-{}", link.url)}
                                class="btn btn-quiet"
                                href={url}
                                target="_blank"
                                rel="noopener noreferrer"
                                onclick={Callback::from(move |_| track_link_click(&label))}
                            >
                                { link.label.clone() }
                            </a>
                        }
                    }) }
                </div>
            }
        }
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
