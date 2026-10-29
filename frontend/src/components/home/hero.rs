use crate::routes::AppRoutes;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Hero)]
pub fn hero() -> Html {
    html! {
        <section class="hero">
            <div class="hero-body">
                <p class="kicker">
                    { "@jheffmedia" }
                </p>
                <h1 class="display">
                    { "Automotive media & pursuit" }
                </h1>
                <p class="lede">
                    { "Track instruction, race data engineering, social content, action-camera runs, and restoration projects." }
                </p>
                <div class="mt-8 flex flex-wrap gap-3">
                    <Link<AppRoutes>
                        to={AppRoutes::Content}
                        classes="btn btn-primary btn-lg"
                    >
                        { "Explore content" }
                    </Link<AppRoutes>>
                    <Link<AppRoutes>
                        to={AppRoutes::Social}
                        classes="btn btn-quiet btn-lg"
                    >
                        { "Social feed" }
                    </Link<AppRoutes>>
                </div>
            </div>
        </section>
    }
}
