use crate::{
    components::ui::{Heading, HeadingLevel, NavLink, Text, TextTone},
    context::AuthContext,
    routes::AppRoutes,
};
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Profile)]
pub fn profile() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let navigator = use_navigator().unwrap();

    match &auth_ctx.auth {
        Some(auth) => html! {
            <div class="space-y-8">
                <div class="rounded-2xl border border-border bg-surface-elevated p-6">
                    <Heading level={HeadingLevel::H1}>
                        { html! { { format!("Welcome, {}", auth.user.username) } } }
                    </Heading>
                    <div class="mt-4 space-y-1 text-sm text-muted">
                        <p>{ format!("Email: {}", auth.user.email) }</p>
                    </div>
                </div>

                <section class="space-y-3">
                    <h2 class="text-lg font-semibold text-foreground">{ "Your account" }</h2>
                    <Text tone={TextTone::Muted}>
                        { "This hub is for @jheffmedia — track instruction, race data, media shoots, and automotive projects. Your account lets you sign in for future client galleries and booking follow-ups." }
                    </Text>
                </section>

                <section class="space-y-3">
                    <h2 class="text-lg font-semibold text-foreground">{ "Quick links" }</h2>
                    <div class="flex flex-wrap gap-2">
                        <NavLink route={AppRoutes::Content} label="Content" />
                        <NavLink route={AppRoutes::Account} label="Account" />
                        <NavLink route={AppRoutes::Profile} label="Profile" />
                        <NavLink route={AppRoutes::Booking} label="Booking" />
                        <NavLink route={AppRoutes::Contact} label="Contact" />
                        <NavLink route={AppRoutes::Galleries} label="Galleries" />
                        <NavLink route={AppRoutes::Shoots} label="Shoots" />
                        <NavLink route={AppRoutes::Social} label="Social" />
                        <NavLink route={AppRoutes::Schedule} label="Schedule" />
                    </div>
                </section>

                <button
                    type="button"
                    class="rounded-lg border border-border px-4 py-2 text-sm text-foreground transition hover:border-accent hover:text-accent"
                    onclick={{
                        let auth_ctx = auth_ctx.clone();
                        Callback::from(move |_| auth_ctx.logout.emit(()))
                    }}
                >
                    { "Log out" }
                </button>
            </div>
        },
        None => {
            navigator.push(&AppRoutes::Home);
            html! {
                <Text tone={TextTone::Muted}>{ "Sign in to view your profile." }</Text>
            }
        }
    }
}
