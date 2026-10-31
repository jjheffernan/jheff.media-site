use crate::components::{
    admin::{ContactAdmin, ContentAdmin, LinksAdmin},
    ui::{Heading, HeadingLevel, Text, TextTone},
};
use crate::context::AuthContext;
use yew::prelude::*;

#[function_component(Admin)]
pub fn admin() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let body = match &auth_ctx.auth {
        None => html! {
            <Text tone={TextTone::Muted}>
                { "Sign in with an admin account to open the dashboard." }
            </Text>
        },
        Some(auth) if auth.user.role != "admin" => html! {
            <Text tone={TextTone::Muted}>{ "This dashboard is for admin accounts." }</Text>
        },
        Some(_) => html! {
            <>
                <LinksAdmin />
                <ContactAdmin />
                <ContentAdmin />
            </>
        },
    };

    html! {
        <div class="space-y-8">
            <Heading
                level={HeadingLevel::H1}
                subtitle="Social links, inbox, content, schedule, and print orders."
            >
                { "Admin" }
            </Heading>
            { body }
        </div>
    }
}
