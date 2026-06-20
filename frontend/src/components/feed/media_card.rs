use crate::components::ui::LazyImage;
use crate::model::FeedItem;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub item: FeedItem,
}

fn is_video(item: &FeedItem) -> bool {
    item.media_type.as_deref() == Some("video")
}

#[function_component(MediaCard)]
pub fn media_card(props: &Props) -> Html {
    let title = props
        .item
        .title
        .clone()
        .unwrap_or_else(|| "Untitled".to_string());
    let video = is_video(&props.item);
    let media_url = props.item.media_url.clone();

    html! {
        <figure class="group relative overflow-hidden rounded-xl border border-border bg-surface-elevated">
            <div class="aspect-[4/3] overflow-hidden bg-surface">
                if video && media_url.is_some() {
                    <video
                        class="h-full w-full object-cover"
                        controls={true}
                        preload="metadata"
                        poster={props.item.thumbnail_url.clone()}
                    >
                        <source src={media_url.unwrap_or_default()} />
                    </video>
                } else {
                    <LazyImage src={props.item.thumbnail_url.clone()} alt={title.clone()} />
                }
            </div>
            <figcaption class="flex items-center gap-2 px-3 py-2 text-xs text-muted">
                if video {
                    <span class="rounded bg-accent/15 px-1.5 py-0.5 font-medium text-accent">
                        { "Video" }
                    </span>
                }
                <span>{ title }</span>
            </figcaption>
        </figure>
    }
}
