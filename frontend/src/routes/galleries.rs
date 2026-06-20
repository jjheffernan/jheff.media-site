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

#[function_component(Galleries)]
pub fn galleries() -> Html {
    let state = use_state(|| LoadState::Loading);

    {
        let state = state.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let result = Request::get("/api/galleries").send().await;
                match result {
                    Ok(resp) if resp.ok() => {
                        if let Ok(data) = resp.json::<ContentListResponse>().await {
                            state.set(LoadState::Ready(data));
                        } else {
                            state.set(LoadState::Error("Could not parse galleries.".into()));
                        }
                    }
                    Ok(_) => state.set(LoadState::Error("Galleries request failed.".into())),
                    Err(err) => state.set(LoadState::Error(err.to_string())),
                }
            });
            || ()
        });
    }

    let content = match &*state {
        LoadState::Loading => html! { <Spinner label="Loading galleries…" /> },
        LoadState::Error(msg) => html! {
            <p class="text-sm text-muted">{ format!("Galleries unavailable: {}", msg) }</p>
        },
        LoadState::Ready(data) if data.items.is_empty() => html! {
            <Text tone={TextTone::Muted}>
                { "No galleries configured yet. Set YEW_FULLSTACK_GALLERIES_JSON on the backend." }
            </Text>
        },
        LoadState::Ready(data) => html! {
            <div class="grid gap-4 sm:grid-cols-2">
                { for data.items.iter().map(|item| html! {
                    <ContentSummaryCard
                        key={item.id.clone()}
                        summary={item.clone()}
                        detail_route={AppRoutes::GalleryDetail { id: item.id.clone() }}
                    />
                }) }
            </div>
        },
    };

    html! {
        <div class="space-y-6">
            <Heading
                level={HeadingLevel::H1}
                subtitle="Finished sets and client deliveries — photos and onboard video from track days, shoots, and restoration projects."
            >
                { "Galleries" }
            </Heading>
            { content }
        </div>
    }
}
