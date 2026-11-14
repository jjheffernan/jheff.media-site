use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum BadgeVariant {
    Default,
    Secondary,
    Outline,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub children: Html,
    #[prop_or(BadgeVariant::Default)]
    pub variant: BadgeVariant,
    #[prop_or_default]
    pub class: Classes,
}

#[function_component(Badge)]
pub fn badge(props: &Props) -> Html {
    let variant = match props.variant {
        BadgeVariant::Default => "badge badge-default",
        BadgeVariant::Secondary => "badge badge-secondary",
        BadgeVariant::Outline => "badge badge-outline",
    };
    html! {
        <span class={classes!(variant, props.class.clone())}>
            { props.children.clone() }
        </span>
    }
}
