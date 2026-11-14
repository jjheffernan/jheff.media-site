use crate::components::ui::Skeleton;
use crate::model::MediaFeedResponse;
use crate::routes::AppRoutes;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, PartialEq)]
enum HeroMedia {
    Loading,
    Ready { url: String, alt: String },
    Empty,
}

/// Full-bleed brand hero — first feed still as edge-to-edge plane.
#[function_component(Hero)]
pub fn hero() -> Html {
    let media = use_state(|| HeroMedia::Loading);

    {
        let media = media.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match Request::get("/api/media/feed?limit=1").send().await {
                    Ok(response) if response.ok() => {
                        if let Ok(feed) = response.json::<MediaFeedResponse>().await {
                            if let Some(item) = feed.items.first() {
                                let url = item
                                    .media_url
                                    .clone()
                                    .filter(|u| !u.is_empty())
                                    .unwrap_or_else(|| item.thumbnail_url.clone());
                                let alt = item
                                    .title
                                    .clone()
                                    .or_else(|| item.caption.clone())
                                    .unwrap_or_else(|| "Featured still".into());
                                media.set(HeroMedia::Ready { url, alt });
                                return;
                            }
                        }
                        media.set(HeroMedia::Empty);
                    }
                    _ => media.set(HeroMedia::Empty),
                }
            });
            || ()
        });
    }

    let media_el = match &*media {
        HeroMedia::Loading => html! { <Skeleton class="h-full w-full rounded-none" /> },
        HeroMedia::Ready { url, alt } => html! {
            <img src={url.clone()} alt={alt.clone()} decoding="async" fetchpriority="high" />
        },
        HeroMedia::Empty => html! {
            <div class="h-full w-full" style="background: linear-gradient(145deg, color-mix(in oklch, var(--accent) 28%, var(--bg)), var(--bg));" />
        },
    };

    html! {
        <section class="home-hero" aria-label="jheff.media">
            <div class="home-hero-media">{ media_el }</div>
            <div class="home-hero-shade" aria-hidden="true" />
            <div class="home-hero-body">
                <p class="kicker">{ "Automotive photography" }</p>
                <h1 class="home-hero-brand">{ "jheff.media" }</h1>
                <p class="home-hero-lede">
                    { "Track instruction, race data, social stills, and restoration — shot and delivered with intent." }
                </p>
                <div class="home-hero-cta">
                    <Link<AppRoutes> to={AppRoutes::Content} classes="btn btn-primary btn-lg">
                        { "Content hub" }
                    </Link<AppRoutes>>
                </div>
            </div>
        </section>
    }
}
