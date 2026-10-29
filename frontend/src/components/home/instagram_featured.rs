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
                    <div class="featured-post">
                        if let Some(thumb) = &featured.thumbnail_url {
                            <a
                                href={featured.post_url.clone()}
                                target="_blank"
                                rel="noopener noreferrer"
                                class="featured-thumb"
                            >
                                <LazyImage src={thumb.clone()} alt={title.clone()} />
                            </a>
                        }
                        <div class="featured-body">
                            <p class="kicker">{ "Instagram" }</p>
                            <a
                                href={featured.post_url.clone()}
                                target="_blank"
                                rel="noopener noreferrer"
                                class="featured-title"
                            >
                                { title }
                            </a>
                            <p class="muted">{ format!("See the latest post on {}", featured.handle) }</p>
                            <div class="featured-actions">
                                <a
                                    href={featured.post_url.clone()}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="btn btn-primary"
                                >
                                    { "View post" }
                                </a>
                                <Link<AppRoutes> to={AppRoutes::Social} classes="btn btn-quiet">
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
