use super::feed_card::FeedCard;
use crate::{
    components::ui::{Section, Spinner, Text, TextTone},
    context::AuthContext,
    model::{MediaFeedResponse, PrintSelection, PrintSelectionInput},
    services::auth_header,
};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
enum FeedState {
    Loading,
    Ready(MediaFeedResponse),
    Error(String),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or(24)]
    pub limit: usize,
    #[prop_or_default]
    pub title: Option<String>,
}

#[function_component(MediaFeedScroll)]
pub fn media_feed_scroll(props: &Props) -> Html {
    let state = use_state(|| FeedState::Loading);
    let limit = use_state(|| props.limit);
    let print_ids = use_state(Vec::<String>::new);
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let jwt_for_prints = auth_ctx.auth.as_ref().map(|a| a.jwt.clone());

    let current_limit = *limit;
    {
        let state = state.clone();
        use_effect_with(current_limit, move |limit| {
            let limit = *limit;
            spawn_local(async move {
                let url = format!("/api/media/feed?limit={}", limit);
                match Request::get(&url).send().await {
                    Ok(response) if response.ok() => {
                        if let Ok(feed) = response.json::<MediaFeedResponse>().await {
                            state.set(FeedState::Ready(feed));
                            return;
                        }
                        state.set(FeedState::Error("Could not parse feed response.".into()));
                    }
                    Ok(_) => state.set(FeedState::Error("Feed request failed.".into())),
                    Err(err) => state.set(FeedState::Error(err.to_string())),
                }
            });
            || ()
        });
    }

    {
        let print_ids = print_ids.clone();
        use_effect_with(jwt_for_prints.clone(), move |jwt_opt| {
            if let Some(jwt) = jwt_opt.clone() {
                spawn_local(async move {
                    let auth_val = auth_header(&jwt);
                    let result = Request::get("/api/account/prints")
                        .header("Authorization", auth_val.as_str())
                        .send()
                        .await;
                    if let Ok(resp) = result {
                        if resp.ok() {
                            if let Ok(items) = resp.json::<Vec<PrintSelection>>().await {
                                print_ids.set(items.into_iter().map(|p| p.item_id).collect());
                            }
                        }
                    }
                });
            }
            || ()
        });
    }

    let on_select_print = auth_ctx.auth.as_ref().map(|auth| {
        let jwt = auth.jwt.clone();
        let print_ids = print_ids.clone();
        Callback::from(move |item: crate::model::MediaFeedItem| {
            let jwt = jwt.clone();
            if print_ids.iter().any(|id| id == &item.id) {
                return;
            }
            let body = PrintSelectionInput {
                item_id: item.id.clone(),
                source: item.source.clone(),
                title: item.title.clone(),
                thumbnail_url: item.thumbnail_url.clone(),
                media_url: item.media_url.clone(),
            };
            let print_ids = print_ids.clone();
            let item_id = item.id.clone();
            spawn_local(async move {
                if let Ok(json) = serde_json::to_string(&body) {
                    let auth_val = auth_header(&jwt);
                    if let Ok(req) = Request::post("/api/account/prints")
                        .header("Authorization", auth_val.as_str())
                        .header("Content-Type", "application/json")
                        .body(json)
                    {
                        if req.send().await.is_ok() {
                            print_ids.set({
                                let mut ids = (*print_ids).clone();
                                ids.push(item_id);
                                ids
                            });
                        }
                    }
                }
            });
        })
    });

    let load_more = {
        let limit = limit.clone();
        let state = state.clone();
        Callback::from(move |_| {
            let next = (*limit).saturating_add(12);
            limit.set(next);
            state.set(FeedState::Loading);
        })
    };

    let content = match &*state {
        FeedState::Loading => html! { <Spinner label="Loading feed…" /> },
        FeedState::Error(message) => html! {
            <p class="text-sm text-muted">{ format!("Feed unavailable: {}", message) }</p>
        },
        FeedState::Ready(feed) if feed.items.is_empty() => html! {
            <Text tone={TextTone::Muted}>
                { "No media yet. Configure Instagram, YouTube, galleries, or shoots on the backend." }
            </Text>
        },
        FeedState::Ready(feed) => {
            let sources = feed.sources.join(", ");
            html! {
                <div class="space-y-6">
                    <p class="text-xs text-muted">
                        { format!("Sources: {}", sources) }
                    </p>
                    <div class="columns-1 gap-4 sm:columns-2 lg:columns-3">
                        { for feed.items.iter().map(|item| html! {
                            <FeedCard
                                key={item.id.clone()}
                                item={item.clone()}
                                on_select_print={on_select_print.clone()}
                                print_selected={print_ids.iter().any(|id| id == &item.id)}
                            />
                        }) }
                    </div>
                    if feed.items.len() >= *limit {
                        <div class="flex justify-center pt-4">
                            <button
                                type="button"
                                class="rounded-lg border border-border px-5 py-2 text-sm font-medium text-foreground transition hover:border-accent hover:text-accent"
                                onclick={load_more}
                            >
                                { "Load more" }
                            </button>
                        </div>
                    }
                </div>
            }
        },
    };

    html! {
        <Section title={props.title.clone()}>
            { content }
        </Section>
    }
}
