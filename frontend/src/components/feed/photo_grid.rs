use super::media_card::MediaCard;
use crate::{
    context::AuthContext,
    model::{FeedItem, PrintSelectionInput},
    services::auth_header,
};
use gloo_net::http::Request;
use std::collections::HashSet;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub items: Vec<FeedItem>,
    #[prop_or_default]
    pub class: String,
    pub collection_id: String,
    pub collection_kind: String,
    pub collection_title: String,
}

fn is_video(item: &FeedItem) -> bool {
    item.media_type
        .as_deref()
        .is_some_and(|kind| kind.to_ascii_lowercase().contains("video"))
}

#[function_component(PhotoGrid)]
pub fn photo_grid(props: &Props) -> Html {
    let selected = use_state(Vec::<String>::new);
    let inflight = use_mut_ref(HashSet::<String>::new);
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let signed_in = auth_ctx.auth.is_some();

    let on_select = {
        let jwt = auth_ctx.auth.as_ref().map(|auth| auth.jwt.clone());
        let selected = selected.clone();
        let inflight = inflight.clone();
        let collection_id = props.collection_id.clone();
        let collection_kind = props.collection_kind.clone();
        let collection_title = props.collection_title.clone();
        Callback::from(move |item: FeedItem| {
            let Some(jwt) = jwt.clone() else {
                return;
            };
            if selected.iter().any(|id| id == &item.id) {
                return;
            }
            if !inflight.borrow_mut().insert(item.id.clone()) {
                return;
            }
            let body = PrintSelectionInput {
                item_id: item.id.clone(),
                source: collection_kind.clone(),
                title: item.title.clone(),
                thumbnail_url: item.thumbnail_url.clone(),
                media_url: item.media_url.clone(),
                collection_id: Some(collection_id.clone()),
                collection_kind: Some(collection_kind.clone()),
                collection_title: Some(collection_title.clone()),
            };
            let selected = selected.clone();
            let inflight = inflight.clone();
            let item_id = item.id.clone();
            spawn_local(async move {
                let posted = post_print(&jwt, &body).await;
                if posted {
                    let mut ids = (*selected).clone();
                    if !ids.iter().any(|id| id == &item_id) {
                        ids.push(item_id);
                        selected.set(ids);
                    }
                } else {
                    inflight.borrow_mut().remove(&item_id);
                }
            });
        })
    };

    html! {
        <div class={format!(
            "grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 {}",
            props.class
        )}>
            { for props.items.iter().map(|item| {
                let control = if is_video(item) {
                    html! {}
                } else if signed_in {
                    let is_selected = selected.iter().any(|id| id == &item.id);
                    let item_for_click = item.clone();
                    let on_select = on_select.clone();
                    html! {
                        <button
                            type="button"
                            disabled={is_selected}
                            class={if is_selected {
                                "rounded-md bg-accent px-2 py-1 text-xs font-medium text-zinc-950"
                            } else {
                                "rounded-md border border-border px-2 py-1 text-xs font-medium text-foreground transition hover:border-accent hover:text-accent"
                            }}
                            onclick={Callback::from(move |_| on_select.emit(item_for_click.clone()))}
                        >
                            { if is_selected { "Selected for print" } else { "Select for print" } }
                        </button>
                    }
                } else {
                    html! {
                        <p class="text-xs text-muted">{ "Sign in to select for print" }</p>
                    }
                };
                html! {
                    <div key={item.id.clone()} class="space-y-2">
                        <MediaCard item={item.clone()} />
                        if item.featured {
                            <p class="muted">{ "Featured" }</p>
                        }
                        { control }
                    </div>
                }
            }) }
        </div>
    }
}

async fn post_print(jwt: &str, body: &PrintSelectionInput) -> bool {
    let Ok(json) = serde_json::to_string(body) else {
        return false;
    };
    let auth_val = auth_header(jwt);
    let Ok(req) = Request::post("/api/account/prints")
        .header("Authorization", auth_val.as_str())
        .header("Content-Type", "application/json")
        .body(json)
    else {
        return false;
    };
    match req.send().await {
        Ok(resp) => resp.status() == 200,
        Err(_) => false,
    }
}
