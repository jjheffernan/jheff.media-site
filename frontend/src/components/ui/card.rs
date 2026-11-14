use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub class: String,
    #[prop_or_default]
    pub title: Option<String>,
    #[prop_or(false)]
    pub flat: bool,
}

#[function_component(Card)]
pub fn card(props: &Props) -> Html {
    let class = format!(
        "card {} {}",
        if props.flat { "card-flat" } else { "" },
        props.class
    );
    html! {
        <article class={class}>
            if let Some(title) = &props.title {
                <h2 class="title-sm">{ title }</h2>
            }
            <div class={if props.title.is_some() { "mt-2" } else { "" }}>
                { for props.children.iter() }
            </div>
        </article>
    }
}
