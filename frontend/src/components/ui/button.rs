use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
    Destructive,
    /// Alias for Outline (legacy call sites).
    Quiet,
}

#[derive(Clone, PartialEq)]
pub enum ButtonSize {
    Sm,
    Md,
    Lg,
}

impl ButtonVariant {
    fn class(&self) -> &'static str {
        match self {
            Self::Primary => "btn btn-primary",
            Self::Secondary => "btn btn-secondary",
            Self::Outline | Self::Quiet => "btn btn-outline",
            Self::Ghost => "btn btn-ghost",
            Self::Destructive => "btn btn-destructive",
        }
    }
}

impl ButtonSize {
    fn class(&self) -> &'static str {
        match self {
            Self::Sm => "btn-sm",
            Self::Md => "",
            Self::Lg => "btn-lg",
        }
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub children: Children,
    #[prop_or_default]
    pub class: String,
    #[prop_or(ButtonVariant::Primary)]
    pub variant: ButtonVariant,
    #[prop_or(ButtonSize::Md)]
    pub size: ButtonSize,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub onclick: Option<Callback<MouseEvent>>,
    #[prop_or("button".into())]
    pub button_type: String,
}

#[function_component(Button)]
pub fn button(props: &Props) -> Html {
    let size = props.size.class();
    let class = format!(
        "{} {} {}",
        props.variant.class(),
        size,
        props.class
    );

    html! {
        <button
            type={props.button_type.clone()}
            class={class}
            disabled={props.disabled}
            onclick={props.onclick.clone()}
        >
            { for props.children.iter() }
        </button>
    }
}
