use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub src: String,
    #[prop_or_default]
    pub alt: String,
    #[prop_or_default]
    pub class: String,
}

#[function_component(LazyImage)]
pub fn lazy_image(props: &Props) -> Html {
    html! {
        <img
            src={props.src.clone()}
            alt={props.alt.clone()}
            loading="lazy"
            decoding="async"
            class={format!(
                "h-full w-full object-cover transition duration-300 group-hover:scale-[1.02] {}",
                props.class
            )}
        />
    }
}
