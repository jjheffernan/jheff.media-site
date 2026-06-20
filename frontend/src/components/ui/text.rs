use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum TextTone {
    Default,
    Muted,
    Accent,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub children: Children,
    #[prop_or_default]
    pub class: String,
    #[prop_or(TextTone::Default)]
    pub tone: TextTone,
}

#[function_component(Text)]
pub fn text(props: &Props) -> Html {
    let tone = match props.tone {
        TextTone::Default => "text-foreground",
        TextTone::Muted => "text-muted",
        TextTone::Accent => "text-accent",
    };

    html! {
        <p class={format!("text-sm {} {}", tone, props.class)}>
            { for props.children.iter() }
        </p>
    }
}
