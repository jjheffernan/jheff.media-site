use crate::routes::AppRoutes;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Hero)]
pub fn hero() -> Html {
    html! {
        <section class="relative overflow-hidden rounded-3xl border border-border bg-surface-elevated">
            <div class="absolute inset-0 bg-gradient-to-br from-accent/15 via-transparent to-transparent" />
            <div class="absolute -right-16 -top-16 h-64 w-64 rounded-full bg-accent/10 blur-3xl" />
            <div class="relative z-10 px-6 py-12 sm:px-10 sm:py-16 lg:px-14 lg:py-20">
                <p class="text-xs font-semibold uppercase tracking-[0.25em] text-accent">
                    { "@jheffmedia" }
                </p>
                <h1 class="mt-4 max-w-3xl text-4xl font-semibold tracking-tight text-foreground sm:text-5xl lg:text-6xl">
                    { "Automotive media & pursuit" }
                </h1>
                <p class="mt-4 max-w-2xl text-base text-muted sm:text-lg">
                    { "Track instruction, race data engineering, social content, action-camera runs, and restoration projects." }
                </p>
                <div class="mt-8 flex flex-wrap gap-3">
                    <Link<AppRoutes>
                        to={AppRoutes::Content}
                        classes="inline-flex items-center rounded-lg bg-accent px-5 py-2.5 text-sm font-medium text-zinc-950 transition hover:bg-accent-hover"
                    >
                        { "Explore content" }
                    </Link<AppRoutes>>
                    <Link<AppRoutes>
                        to={AppRoutes::Social}
                        classes="inline-flex items-center rounded-lg border border-border bg-surface px-5 py-2.5 text-sm font-medium text-foreground transition hover:border-accent hover:text-accent"
                    >
                        { "Social feed" }
                    </Link<AppRoutes>>
                </div>
            </div>
        </section>
    }
}
