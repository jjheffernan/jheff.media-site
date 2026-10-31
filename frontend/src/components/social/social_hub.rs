use super::post_card::PostCard;
use crate::{
    components::{
        home::InstagramFeaturedCard,
        ui::{Grid, Heading, HeadingLevel, Spinner, Text, TextTone},
    },
    model::SocialHubResponse,
    routes::AppRoutes,
};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

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

    let feed_content = match &*state {
        LoadState::Loading => html! { <Spinner label="Loading social feed…" /> },
        LoadState::Error(msg) => html! {
            <p class="text-sm text-muted">{ format!("Social feed unavailable: {}", msg) }</p>
        },
        LoadState::Ready(hub) => {
            let instagram_posts: Vec<_> = hub
                .posts
                .iter()
                .filter(|p| p.platform == "instagram")
                .cloned()
                .collect();
            let youtube_posts: Vec<_> = hub
                .posts
                .iter()
                .filter(|p| p.platform == "youtube")
                .cloned()
                .collect();

            if instagram_posts.is_empty() && youtube_posts.is_empty() {
                html! {
                    <Text tone={TextTone::Muted}>
                        { "Instagram and YouTube appear here when the backend keys are set." }
                    </Text>
                }
            } else {
                html! {
                    <div class="space-y-10">
                        if !instagram_posts.is_empty() {
                            <section class="space-y-4">
                                <Heading level={HeadingLevel::H2} subtitle="Recent posts from @jheffmedia on Instagram.">
                                    { "Instagram" }
                                </Heading>
                                <Grid cols_sm={1} cols_lg={3}>
                                    { for instagram_posts.iter().map(|post| html! {
                                        <PostCard key={post.id.clone()} post={post.clone()} />
                                    }) }
                                </Grid>
                            </section>
                        }

                        if !youtube_posts.is_empty() {
                            <section class="space-y-4">
                                <Heading level={HeadingLevel::H2} subtitle="Latest videos from @jheffmedia on YouTube.">
                                    { "YouTube" }
                                </Heading>
                                <Grid cols_sm={1} cols_lg={3}>
                                    { for youtube_posts.iter().map(|post| html! {
                                        <PostCard key={post.id.clone()} post={post.clone()} />
                                    }) }
                                </Grid>
                            </section>
                        }
                    </div>
                }
            }
        }
    };

    html! {
        <div class="space-y-8">
            <Heading
                level={HeadingLevel::H1}
                subtitle="Instagram, YouTube, and cross-platform posts from @jheffmedia."
            >
                { "Social" }
            </Heading>
            <Link<AppRoutes> to={AppRoutes::Links} classes="btn btn-quiet">
                { "All social links" }
            </Link<AppRoutes>>

            <InstagramFeaturedCard />

            { feed_content }
        </div>
    }
}
