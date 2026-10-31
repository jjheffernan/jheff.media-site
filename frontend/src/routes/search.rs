use crate::components::ui::{Heading, HeadingLevel, Spinner, Text, TextTone};
use crate::routes::AppRoutes;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SearchHit {
    kind: String,
    id: String,
    title: String,
    detail: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct SearchResponse {
    query: String,
    hits: Vec<SearchHit>,
}

#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Ready(SearchResponse),
    Error(String),
}

fn hit_route(hit: &SearchHit) -> AppRoutes {
    match hit.kind.as_str() {
        "gallery" => AppRoutes::GalleryDetail { id: hit.id.clone() },
        "shoot" => AppRoutes::ShootDetail { id: hit.id.clone() },
        _ => AppRoutes::Schedule,
    }
}

fn kind_label(kind: &str) -> &'static str {
    match kind {
        "gallery" => "Gallery",
        "shoot" => "Shoot",
        _ => "Date",
    }
}

#[function_component(Search)]
pub fn search() -> Html {
    let location = use_location();
    let query = location
        .and_then(|loc| loc.query::<SearchQuery>().ok())
        .map(|query| query.q)
        .unwrap_or_default();
    let state = use_state(|| LoadState::Loading);

    {
        let state = state.clone();
        use_effect_with(query.clone(), move |query| {
            let query = query.clone();
            if query.chars().count() < 2 {
                state.set(LoadState::Ready(SearchResponse {
                    query,
                    hits: vec![],
                }));
            } else {
                state.set(LoadState::Loading);
                spawn_local(async move {
                    let url = format!("/api/search?q={}", js_sys::encode_uri_component(&query));
                    let result = Request::get(&url).send().await;
                    match result {
                        Ok(resp) if resp.ok() => match resp.json::<SearchResponse>().await {
                            Ok(body) => state.set(LoadState::Ready(body)),
                            Err(_) => state.set(LoadState::Error("Could not read search results.".into())),
                        },
                        Ok(_) => state.set(LoadState::Error("Search failed.".into())),
                        Err(err) => state.set(LoadState::Error(err.to_string())),
                    }
                });
            }
            || ()
        });
    }

    let body = match &*state {
        LoadState::Loading => html! { <Spinner label="Searching…" /> },
        LoadState::Error(msg) => html! { <p class="muted">{ msg }</p> },
        LoadState::Ready(data) if data.query.chars().count() < 2 => html! {
            <Text tone={TextTone::Muted}>{ "Search galleries, shoots, and dates." }</Text>
        },
        LoadState::Ready(data) if data.hits.is_empty() => html! {
            <p class="muted">{ format!("No matches for {}.", data.query) }</p>
        },
        LoadState::Ready(data) => html! {
            <ul class="search-results">
                { for data.hits.iter().map(|hit| html! {
                    <li key={format!("{}-{}-{}", hit.kind, hit.id, hit.title)}>
                        <Link<AppRoutes> to={hit_route(hit)} classes="card-link">
                            <p class="kicker">{ kind_label(&hit.kind) }</p>
                            <p class="title-sm">{ hit.title.clone() }</p>
                            if let Some(detail) = &hit.detail {
                                <p class="muted">{ detail.clone() }</p>
                            }
                        </Link<AppRoutes>>
                    </li>
                }) }
            </ul>
        },
    };

    html! {
        <div class="space-y-6">
            <Heading level={HeadingLevel::H1}>{ "Search" }</Heading>
            { body }
        </div>
    }
}
