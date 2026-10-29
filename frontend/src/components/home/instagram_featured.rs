use crate::{
    components::ui::{LazyImage, Spinner},
    model::InstagramFeatured,
    routes::AppRoutes,
};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Ready(InstagramFeatured),
    Hidden,
}

#[function_component(InstagramFeaturedCard)]
pub fn instagram_featured_card() -> Html {
    let state = use_state(|| LoadState::Loading);

    {
        let state = state.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match Request::get("/api/media/instagram/featured").send().await {
                    Ok(resp) if resp.status() == 200 => {
                        if let Ok(featured) = resp.json::<InstagramFeatured>().await {
                            state.set(LoadState::Ready(featured));
                            return;
                        }
                        state.set(LoadState::Hidden);
                    }
                    _ => state.set(LoadState::Hidden),
                }
            });
            || ()
        });
    }

    match &*state {
        LoadState::Loading => html! { <Spinner label="Loading featured post…" /> },
        LoadState::Hidden => html! {},
        LoadState::Ready(featured) => {
            let title = featured
                .title
                .clone()
                .unwrap_or_else(|| "Latest from Instagram".to_string());
            html! {
                <section class="card card-flat overflow-hidden">
                    <div class="flex flex-col gap-4 p-5 sm:flex-row sm:items-center">
                        if let Some(thumb) = &featured.thumbnail_url {
                            <a
                                href={featured.post_url.clone()}
                                target="_blank"
                                rel="noopener noreferrer"
                                class="block shrink-0 overflow-hidden rounded-xl sm:w-48"
                            >
                                <LazyImage src={thumb.clone()} alt={title.clone()} />
                            </a>
                        }
                        <div class="flex-1 space-y-3">
                            <p class="kicker">
                                { "Instagram" }
                            </p>
                            <a
                                href={featured.post_url.clone()}
                                target="_blank"
                                rel="noopener noreferrer"
                                class="text-xl font-semibold text-foreground hover:text-accent"
                            >
                                { title }
                            </a>
                            <p class="text-sm text-muted">
                                { format!("See the latest post on {}", featured.handle) }
                            </p>
                            <div class="flex flex-wrap gap-3">
                                <a
                                    href={featured.post_url.clone()}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="btn btn-primary"
                                >
                                    { "View post" }
                                </a>
                                <Link<AppRoutes>
                                    to={AppRoutes::Social}
                                    classes="btn btn-quiet"
                                >
                                    { "All social →" }
                                </Link<AppRoutes>>
                            </div>
                        </div>
                    </div>
                </section>
            }
        }
    }
}
