use crate::components::ui::LazyImage;
use crate::model::FeedItem;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub item: FeedItem,
}

#[function_component(PhotoCard)]
pub fn photo_card(props: &Props) -> Html {
    let title = props
        .item
        .title
        .clone()
        .unwrap_or_else(|| "Untitled".to_string());

    html! {
        <figure class="group relative overflow-hidden rounded-xl border border-zinc-800 bg-zinc-900/40">
            <div class="aspect-[4/3] overflow-hidden bg-zinc-900">
                <LazyImage src={props.item.thumbnail_url.clone()} alt={title.clone()} />
            </div>
            <figcaption class="px-3 py-2 text-xs text-zinc-400">
                { title }
            </figcaption>
        </figure>
    }
}
