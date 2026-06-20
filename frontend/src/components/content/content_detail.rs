use crate::{
    components::{
        feed::PhotoGrid,
        ui::{Heading, HeadingLevel, Spinner, Text, TextTone},
    },
    model::{ContentCollection, FeedItem, ServerResponse},
};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub id: String,
    pub api_path: String,
    pub back_route: crate::routes::AppRoutes,
    pub back_label: String,
}

#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Ready(ContentCollection),
    Error(String),
}

#[function_component(ContentDetail)]
pub fn content_detail(props: &Props) -> Html {
    let state = use_state(|| LoadState::Loading);
    let api_url = format!("{}/{}", props.api_path, props.id);

    {
        let state = state.clone();
        let api_url = api_url.clone();
        use_effect_with((api_url.clone(), props.id.clone()), move |_| {
            spawn_local(async move {
                let result = Request::get(&api_url).send().await;
                match result {
                    Ok(resp) if resp.ok() => {
                        if let Ok(collection) = resp.json::<ContentCollection>().await {
                            state.set(LoadState::Ready(collection));
                            return;
                        }
                        state.set(LoadState::Error("Could not parse content.".into()));
                    }
                    Ok(resp) => {
                        if let Ok(body) = resp.json::<ServerResponse<String>>().await {
                            state.set(LoadState::Error(body.message));
                        } else {
                            state.set(LoadState::Error("Content not found.".into()));
                        }
                    }
                    Err(err) => state.set(LoadState::Error(err.to_string())),
                }
            });
            || ()
        });
    }

    let body = match &*state {
        LoadState::Loading => html! { <Spinner label="Loading…" /> },
        LoadState::Error(msg) => html! {
            <p class="text-sm text-muted">{ msg.clone() }</p>
        },
        LoadState::Ready(collection) => {
            let feed_items: Vec<FeedItem> = collection
                .media
                .iter()
                .map(|m| FeedItem {
                    id: m.id.clone(),
                    title: m.title.clone(),
                    thumbnail_url: m.thumbnail_url.clone(),
                    media_url: m.media_url.clone(),
                    media_type: m.media_type.clone(),
                    width: m.width,
                    height: m.height,
                })
                .collect();

            let meta = {
                let mut parts = vec![];
                if let Some(date) = &collection.date {
                    parts.push(date.clone());
                }
                if let Some(location) = &collection.location {
                    parts.push(location.clone());
                }
                if let Some(status) = &collection.status {
                    parts.push(status.clone());
                }
                parts.join(" · ")
            };

            html! {
                <div class="space-y-6">
                    if let Some(summary) = &collection.summary {
                        <p class="text-sm text-muted">{ summary.clone() }</p>
                    }
                    if !meta.is_empty() {
                        <p class="text-sm text-muted">{ meta }</p>
                    }
                    if feed_items.is_empty() {
                        <Text tone={TextTone::Muted}>
                            { "No media in this collection yet." }
                        </Text>
                    } else {
                        <PhotoGrid items={feed_items} />
                    }
                </div>
            }
        },
    };

    let title = match &*state {
        LoadState::Ready(c) => c.title.clone(),
        _ => props.id.clone(),
    };

    html! {
        <div class="space-y-6">
            <div class="space-y-2">
                <Link<crate::routes::AppRoutes>
                    to={props.back_route.clone()}
                    classes="text-sm text-muted transition hover:text-accent"
                >
                    { format!("← Back to {}", props.back_label) }
                </Link<crate::routes::AppRoutes>>
                <Heading level={HeadingLevel::H1}>
                    { html! { { title.clone() } } }
                </Heading>
            </div>
            { body }
        </div>
    }
}
