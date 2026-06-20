use super::photo_grid::PhotoGrid;
use crate::components::ui::{Section, Spinner, Text, TextTone};
use crate::model::{FeedResponse, ServerResponse};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
enum FeedState {
    Loading,
    Ready(FeedResponse),
    Error(String),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or(24)]
    pub limit: usize,
    #[prop_or_default]
    pub title: Option<String>,
}

#[function_component(PhotoFeed)]
pub fn photo_feed(props: &Props) -> Html {
    let state = use_state(|| FeedState::Loading);
    let limit = props.limit;

    {
        let state = state.clone();
        use_effect_with(limit, move |limit| {
            let limit = *limit;
            spawn_local(async move {
                let url = format!("/api/feed?limit={}", limit);
                let result = Request::get(&url).send().await;

                match result {
                    Ok(response) if response.ok() => {
                        if let Ok(feed) = response.json::<FeedResponse>().await {
                            state.set(FeedState::Ready(feed));
                            return;
                        }
                        state.set(FeedState::Error("Could not parse feed response.".into()));
                    }
                    Ok(response) => {
                        if let Ok(body) = response.json::<ServerResponse<String>>().await {
                            state.set(FeedState::Error(body.message));
                        } else {
                            state.set(FeedState::Error("Feed request failed.".into()));
                        }
                    }
                    Err(err) => state.set(FeedState::Error(err.to_string())),
                }
            });
            || ()
        });
    }

    let content = match &*state {
        FeedState::Loading => html! { <Spinner label="Loading photos…" /> },
        FeedState::Error(message) => html! {
            <p class="text-sm text-zinc-400">
                { format!("Feed unavailable: {}", message) }
            </p>
        },
        FeedState::Ready(feed) if feed.items.is_empty() => html! {
            <Text tone={TextTone::Muted}>
                { "No photos configured yet. Set YEW_FULLSTACK_FEED_PROVIDER to immich or static on the backend." }
            </Text>
        },
        FeedState::Ready(feed) => html! {
            <div>
                <p class="mb-3 text-sm text-zinc-400">
                    { format!("Source: {}", feed.source) }
                </p>
                <PhotoGrid items={feed.items.clone()} />
            </div>
        },
    };

    html! {
        <Section title={props.title.clone()}>
            { content }
        </Section>
    }
}
