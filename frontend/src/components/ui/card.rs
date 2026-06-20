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

#[function_component(Card)]
pub fn card(props: &Props) -> Html {
    html! {
        <article class={format!(
            "rounded-2xl border border-border bg-surface-elevated p-6 shadow-sm shadow-black/5 dark:shadow-black/20 {}",
            props.class
        )}>
            if let Some(title) = &props.title {
                <h2 class="text-lg font-semibold text-foreground">{ title }</h2>
            }
            <div class={if props.title.is_some() { "mt-2" } else { "" }}>
                { for props.children.iter() }
            </div>
        </article>
    }
}
