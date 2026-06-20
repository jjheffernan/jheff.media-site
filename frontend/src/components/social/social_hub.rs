use super::post_card::PostCard;
use crate::{
    components::ui::{Grid, Heading, HeadingLevel, Spinner, Text, TextTone},
    model::SocialHubResponse,
};
use gloo_net::http::Request;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlScriptElement;
use yew::prelude::*;

const INSTAGRAM_PROFILE: &str = "https://www.instagram.com/jheffmedia/";
const INSTAGRAM_EMBED_SCRIPT: &str = "https://www.instagram.com/embed.js";

#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Ready(SocialHubResponse),
    Error(String),
}

#[function_component(SocialHub)]
pub fn social_hub() -> Html {
    let state = use_state(|| LoadState::Loading);

    {
        let state = state.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let result = Request::get("/api/social/hub").send().await;
                match result {
                    Ok(resp) if resp.ok() => {
                        if let Ok(hub) = resp.json::<SocialHubResponse>().await {
                            state.set(LoadState::Ready(hub));
                            return;
                        }
                        state.set(LoadState::Error("Could not parse social feed.".into()));
                    }
                    Ok(_) => state.set(LoadState::Error("Social feed request failed.".into())),
                    Err(err) => state.set(LoadState::Error(err.to_string())),
                }
            });
            || ()
        });
    }

  {
        let state = state.clone();
        use_effect_with(state, move |state| {
            if let LoadState::Ready(hub) = &**state {
                if hub.posts.is_empty() {
                    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                        if document.get_element_by_id("instagram-embed-script").is_none() {
                            if let Ok(script) = document.create_element("script") {
                                let script: HtmlScriptElement = script.unchecked_into();
                                script.set_id("instagram-embed-script");
                                script.set_src(INSTAGRAM_EMBED_SCRIPT);
                                script.set_async(true);
                                let _ = document.body().and_then(|body| body.append_child(&script).ok());
                            }
                        }
                    }
                }
            }
            || ()
        });
    }

    let content = match &*state {
        LoadState::Loading => html! { <Spinner label="Loading Instagram feed…" /> },
        LoadState::Error(msg) => html! {
            <p class="text-sm text-muted">{ format!("Social feed unavailable: {}", msg) }</p>
        },
        LoadState::Ready(hub) if hub.posts.is_empty() => html! {
            <div class="space-y-4">
                <Text tone={TextTone::Muted}>
                    { "Live Instagram posts load when YEW_FULLSTACK_INSTAGRAM_ACCESS_TOKEN is set on the backend. Showing profile embed for @jheffmedia." }
                </Text>
                <blockquote
                    class="instagram-media"
                    data-instgrm-permalink={INSTAGRAM_PROFILE}
                    data-instgrm-version="14"
                />
            </div>
        },
        LoadState::Ready(hub) => {
            let source_note = match hub.source.as_str() {
                "instagram" => "Live from Instagram @jheffmedia",
                "config" => "From configured Instagram samples",
                _ => "Instagram @jheffmedia",
            };

            html! {
                <div class="space-y-6">
                    <div class="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-border bg-surface-elevated px-5 py-4">
                        <div>
                            <p class="text-xs font-semibold uppercase tracking-[0.2em] text-accent">
                                { "Instagram" }
                            </p>
                            <a
                                class="text-lg font-semibold text-foreground hover:text-accent"
                                href={INSTAGRAM_PROFILE}
                                target="_blank"
                                rel="noopener noreferrer"
                            >
                                { "@jheffmedia" }
                            </a>
                        </div>
                        <p class="text-xs text-muted">{ source_note }</p>
                    </div>
                    <Grid cols_sm={1} cols_lg={3}>
                        { for hub.posts.iter().map(|post| html! {
                            <PostCard key={post.id.clone()} post={post.clone()} />
                        }) }
                    </Grid>
                </div>
            }
        },
    };

    html! {
        <div class="space-y-6">
            <Heading
                level={HeadingLevel::H1}
                subtitle="Recent posts from @jheffmedia on Instagram — pulled automatically when an access token is configured."
            >
                { "Social" }
            </Heading>
            { content }
        </div>
    }
}
