use super::media_card::MediaCard;
use crate::model::FeedItem;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub items: Vec<FeedItem>,
    #[prop_or_default]
    pub class: String,
}

/// Keyed grid — patterned after yewstack `keyed_list`.
#[function_component(PhotoGrid)]
pub fn photo_grid(props: &Props) -> Html {
    html! {
        <div class={format!(
            "grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 {}",
            props.class
        )}>
            { for props.items.iter().map(|item| html! {
                <MediaCard key={item.id.clone()} item={item.clone()} />
            }) }
        </div>
    }
}
