use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Ghost,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub children: Children,
    #[prop_or_default]
    pub class: String,
    #[prop_or(ButtonVariant::Primary)]
    pub variant: ButtonVariant,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub onclick: Option<Callback<MouseEvent>>,
}

#[function_component(Button)]
pub fn button(props: &Props) -> Html {
    let variant = match props.variant {
        ButtonVariant::Primary => "btn btn-primary",
        ButtonVariant::Ghost => "btn btn-quiet",
    };

    html! {
        <button
            type="button"
            class={format!("{} {}", variant, props.class)}
            disabled={props.disabled}
            onclick={props.onclick.clone()}
        >
            { for props.children.iter() }
        </button>
    }
}
