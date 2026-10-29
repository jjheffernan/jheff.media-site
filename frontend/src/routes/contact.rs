use crate::{
    components::ContactForm,
    components::ui::{Heading, HeadingLevel},
};
use yew::prelude::*;

#[function_component(Contact)]
pub fn contact() -> Html {
    html! {
        <div class="space-y-6">
            <Heading
                level={HeadingLevel::H1}
                subtitle="Questions about track instruction, shoots, race data, or restoration projects — reach out here."
            >
                { "Contact" }
            </Heading>
            <div class="card max-w-2xl">
                <ContactForm />
            </div>
        </div>
    }
}
