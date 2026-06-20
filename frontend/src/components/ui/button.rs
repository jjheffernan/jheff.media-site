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
    let base = "inline-flex items-center justify-center rounded-lg px-4 py-2 text-sm font-medium transition focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-cyan-500 disabled:opacity-50";
    let variant = match props.variant {
        ButtonVariant::Primary => "bg-accent text-zinc-950 hover:bg-accent-hover",
        ButtonVariant::Ghost => "text-foreground hover:bg-surface-elevated",
    };

    html! {
        <button
            type="button"
            class={format!("{} {} {}", base, variant, props.class)}
            disabled={props.disabled}
            onclick={props.onclick.clone()}
        >
            { for props.children.iter() }
        </button>
    }
}
