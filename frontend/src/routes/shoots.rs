use crate::{
    components::{
        content::ContentSummaryCard,
        ui::{Heading, HeadingLevel, Spinner, Text, TextTone},
    },
    model::ContentListResponse,
    routes::AppRoutes,
};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Ready(ContentListResponse),
    Error(String),
}

#[function_component(Shoots)]
pub fn shoots() -> Html {
    let state = use_state(|| LoadState::Loading);

    {
        let state = state.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let result = Request::get("/api/shoots").send().await;
                match result {
                    Ok(resp) if resp.ok() => {
                        if let Ok(data) = resp.json::<ContentListResponse>().await {
                            state.set(LoadState::Ready(data));
                        } else {
                            state.set(LoadState::Error("Could not parse shoots.".into()));
                        }
                    }
                    Ok(_) => state.set(LoadState::Error("Shoots request failed.".into())),
                    Err(err) => state.set(LoadState::Error(err.to_string())),
                }
            });
            || ()
        });
    }

    let content = match &*state {
        LoadState::Loading => html! { <Spinner label="Loading shoots…" /> },
        LoadState::Error(msg) => html! {
            <p class="text-sm text-muted">{ format!("Shoots unavailable: {}", msg) }</p>
        },
        LoadState::Ready(data) if data.items.is_empty() => html! {
            <Text tone={TextTone::Muted}>
                { "No shoots configured yet. Set YEW_FULLSTACK_SHOOTS_JSON on the backend." }
            </Text>
        },
        LoadState::Ready(data) => html! {
            <div class="grid gap-4 sm:grid-cols-2">
                { for data.items.iter().map(|item| html! {
                    <ContentSummaryCard
                        key={item.id.clone()}
                        summary={item.clone()}
                        detail_route={AppRoutes::ShootDetail { id: item.id.clone() }}
                    />
                }) }
            </div>
        },
    };

    html! {
        <div class="space-y-6">
            <Heading
                level={HeadingLevel::H1}
                subtitle="Upcoming and recent sessions — track instruction, dealer inventory, action-camera runs, and private collections."
            >
                { "Shoots" }
            </Heading>
            { content }
        </div>
    }
}
