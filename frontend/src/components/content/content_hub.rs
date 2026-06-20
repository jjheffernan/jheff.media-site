use super::content_summary_card::ContentSummaryCard;
use crate::{
    components::{
        social::PostCard,
        ui::{Grid, Heading, HeadingLevel, Spinner, Text, TextTone},
    },
    model::{ContentListResponse, SocialHubResponse},
    routes::AppRoutes,
};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

const SOCIAL_LIMIT: usize = 3;
const SHOOT_LIMIT: usize = 2;
const GALLERY_LIMIT: usize = 2;

#[derive(Clone, PartialEq)]
enum SliceState<T> {
    Loading,
    Ready(T),
    Error(String),
}

#[function_component(ContentHub)]
pub fn content_hub() -> Html {
    let social = use_state(|| SliceState::<SocialHubResponse>::Loading);
    let shoots = use_state(|| SliceState::<ContentListResponse>::Loading);
    let galleries = use_state(|| SliceState::<ContentListResponse>::Loading);

    {
        let social = social.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let result = Request::get("/api/social/hub").send().await;
                match result {
                    Ok(resp) if resp.ok() => {
                        if let Ok(hub) = resp.json::<SocialHubResponse>().await {
                            social.set(SliceState::Ready(hub));
                        } else {
                            social.set(SliceState::Error("Could not parse social feed.".into()));
                        }
                    }
                    Ok(_) => social.set(SliceState::Error("Social feed unavailable.".into())),
                    Err(err) => social.set(SliceState::Error(err.to_string())),
                }
            });
            || ()
        });
    }

    {
        let shoots = shoots.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let result = Request::get("/api/shoots").send().await;
                match result {
                    Ok(resp) if resp.ok() => {
                        if let Ok(data) = resp.json::<ContentListResponse>().await {
                            shoots.set(SliceState::Ready(data));
                        } else {
                            shoots.set(SliceState::Error("Could not parse shoots.".into()));
                        }
                    }
                    Ok(_) => shoots.set(SliceState::Error("Shoots unavailable.".into())),
                    Err(err) => shoots.set(SliceState::Error(err.to_string())),
                }
            });
            || ()
        });
    }

    {
        let galleries = galleries.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let result = Request::get("/api/galleries").send().await;
                match result {
                    Ok(resp) if resp.ok() => {
                        if let Ok(data) = resp.json::<ContentListResponse>().await {
                            galleries.set(SliceState::Ready(data));
                        } else {
                            galleries.set(SliceState::Error("Could not parse galleries.".into()));
                        }
                    }
                    Ok(_) => galleries.set(SliceState::Error("Galleries unavailable.".into())),
                    Err(err) => galleries.set(SliceState::Error(err.to_string())),
                }
            });
            || ()
        });
    }

    let social_section = match &*social {
        SliceState::Loading => html! { <Spinner label="Loading social feed…" /> },
        SliceState::Error(msg) => html! {
            <p class="text-sm text-muted">{ format!("{}", msg) }</p>
        },
        SliceState::Ready(hub) if hub.posts.is_empty() => html! {
            <Text tone={TextTone::Muted}>{ "No social posts configured yet." }</Text>
        },
        SliceState::Ready(hub) => html! {
            <Grid cols_sm={1} cols_lg={3}>
                { for hub.posts.iter().take(SOCIAL_LIMIT).map(|post| html! {
                    <PostCard key={post.id.clone()} post={post.clone()} />
                }) }
            </Grid>
        },
    };

    let shoots_section = match &*shoots {
        SliceState::Loading => html! { <Spinner label="Loading shoots…" /> },
        SliceState::Error(msg) => html! {
            <p class="text-sm text-muted">{ format!("{}", msg) }</p>
        },
        SliceState::Ready(data) if data.items.is_empty() => html! {
            <Text tone={TextTone::Muted}>{ "No shoots configured yet." }</Text>
        },
        SliceState::Ready(data) => html! {
            <div class="grid gap-4 sm:grid-cols-2">
                { for data.items.iter().take(SHOOT_LIMIT).map(|item| html! {
                    <ContentSummaryCard
                        key={item.id.clone()}
                        summary={item.clone()}
                        detail_route={AppRoutes::ShootDetail { id: item.id.clone() }}
                    />
                }) }
            </div>
        },
    };

    let galleries_section = match &*galleries {
        SliceState::Loading => html! { <Spinner label="Loading galleries…" /> },
        SliceState::Error(msg) => html! {
            <p class="text-sm text-muted">{ format!("{}", msg) }</p>
        },
        SliceState::Ready(data) if data.items.is_empty() => html! {
            <Text tone={TextTone::Muted}>{ "No galleries configured yet." }</Text>
        },
        SliceState::Ready(data) => html! {
            <div class="grid gap-4 sm:grid-cols-2">
                { for data.items.iter().take(GALLERY_LIMIT).map(|item| html! {
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
        <div class="space-y-10">
            <Heading
                level={HeadingLevel::H1}
                subtitle="Social posts, upcoming shoots, and published galleries — your @jheffmedia content in one place."
            >
                { "Content hub" }
            </Heading>

            <section class="space-y-4">
                <div class="flex items-end justify-between gap-4">
                    <Heading level={HeadingLevel::H2} subtitle="Latest from Instagram, YouTube, and cross-platform posts.">
                        { "Social" }
                    </Heading>
                    <Link<AppRoutes>
                        to={AppRoutes::Social}
                        classes="text-sm font-medium text-accent hover:underline"
                    >
                        { "View all social →" }
                    </Link<AppRoutes>>
                </div>
                { social_section }
            </section>

            <section class="space-y-4">
                <div class="flex items-end justify-between gap-4">
                    <Heading level={HeadingLevel::H2} subtitle="Track days, inventory shoots, and action-camera sessions.">
                        { "Shoots" }
                    </Heading>
                    <Link<AppRoutes>
                        to={AppRoutes::Shoots}
                        classes="text-sm font-medium text-accent hover:underline"
                    >
                        { "View all shoots →" }
                    </Link<AppRoutes>>
                </div>
                { shoots_section }
            </section>

            <section class="space-y-4">
                <div class="flex items-end justify-between gap-4">
                    <Heading level={HeadingLevel::H2} subtitle="Finished sets and client deliveries — photos and onboard video.">
                        { "Galleries" }
                    </Heading>
                    <Link<AppRoutes>
                        to={AppRoutes::Galleries}
                        classes="text-sm font-medium text-accent hover:underline"
                    >
                        { "View all galleries →" }
                    </Link<AppRoutes>>
                </div>
                { galleries_section }
            </section>
        </div>
    }
}
