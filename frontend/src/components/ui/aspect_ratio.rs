use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum AspectKind {
    Square,
    Video,
    Photo,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or(AspectKind::Photo)]
    pub kind: AspectKind,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub class: String,
}

#[function_component(AspectRatio)]
pub fn aspect_ratio(props: &Props) -> Html {
    let kind = match props.kind {
        AspectKind::Square => "aspect aspect-square",
        AspectKind::Video => "aspect aspect-video",
        AspectKind::Photo => "aspect aspect-photo",
    };
    html! {
        <div class={format!("{} {}", kind, props.class)}>
            { for props.children.iter() }
        </div>
    }
}
