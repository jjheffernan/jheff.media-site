use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub class: String,
    #[prop_or_default]
    pub title: Option<String>,
}

#[function_component(Section)]
pub fn section(props: &Props) -> Html {
    html! {
        <section class={format!("space-y-4 {}", props.class)}>
            if let Some(title) = &props.title {
                <h2 class="kicker">
                    { title }
                </h2>
            }
            { for props.children.iter() }
        </section>
    }
}
