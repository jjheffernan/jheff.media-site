use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub class: String,
}

#[function_component(Skeleton)]
pub fn skeleton(props: &Props) -> Html {
    html! {
        <div class={format!("skeleton {}", props.class)} aria-hidden="true" />
    }
}
