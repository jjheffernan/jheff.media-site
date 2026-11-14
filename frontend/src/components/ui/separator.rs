use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub class: String,
    #[prop_or(false)]
    pub vertical: bool,
}

#[function_component(Separator)]
pub fn separator(props: &Props) -> Html {
    let base = if props.vertical {
        "separator separator-vertical"
    } else {
        "separator"
    };
    html! {
        <hr class={format!("{} {}", base, props.class)} role="separator" />
    }
}
