use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub title: Option<String>,
    #[prop_or_default]
    pub description: Option<String>,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub class: String,
}

#[function_component(Empty)]
pub fn empty(props: &Props) -> Html {
    html! {
        <div class={format!("empty {}", props.class)}>
            if let Some(title) = &props.title {
                <p class="empty-title">{ title }</p>
            }
            if let Some(description) = &props.description {
                <p class="empty-desc">{ description }</p>
            }
            { for props.children.iter() }
        </div>
    }
}
