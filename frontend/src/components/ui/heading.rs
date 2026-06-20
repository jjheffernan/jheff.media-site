use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub children: Children,
    #[prop_or_default]
    pub class: String,
    #[prop_or(HeadingLevel::H1)]
    pub level: HeadingLevel,
    #[prop_or_default]
    pub subtitle: Option<String>,
}

#[function_component(Heading)]
pub fn heading(props: &Props) -> Html {
    let title = match props.level {
        HeadingLevel::H1 => html! {
            <h1 class="text-3xl font-semibold tracking-tight text-foreground sm:text-4xl">
                { for props.children.iter() }
            </h1>
        },
        HeadingLevel::H2 => html! {
            <h2 class="text-2xl font-semibold tracking-tight text-foreground">
                { for props.children.iter() }
            </h2>
        },
        HeadingLevel::H3 => html! {
            <h3 class="text-lg font-semibold text-foreground">
                { for props.children.iter() }
            </h3>
        },
    };

    html! {
        <div class={props.class.clone()}>
            { title }
            if let Some(subtitle) = &props.subtitle {
                <p class="mt-3 max-w-2xl text-base text-muted">{ subtitle }</p>
            }
        </div>
    }
}
