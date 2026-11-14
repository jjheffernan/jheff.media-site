use crate::{
    components::ui::{Badge, BadgeVariant, LazyImage},
    model::MediaFeedItem,
    routes::AppRoutes,
};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub item: MediaFeedItem,
    #[prop_or_default]
    pub on_select_print: Option<Callback<MediaFeedItem>>,
    #[prop_or_default]
    pub print_selected: bool,
}

fn source_label(source: &str) -> &'static str {
    match source {
        "instagram" => "Instagram",
        "youtube" => "YouTube",
        "upload" => "Gallery",
        _ => "Media",
    }
}

fn is_printable(item: &MediaFeedItem) -> bool {
    item.media_type != "video"
}

#[function_component(FeedCard)]
pub fn feed_card(props: &Props) -> Html {
    let item = &props.item;
    let title = item
        .title
        .clone()
        .unwrap_or_else(|| "Untitled".to_string());
    let video = item.media_type == "video";

    let collection_link = match (&item.collection_kind, &item.collection_id) {
        (Some(kind), Some(id)) if kind == "gallery" => {
            Some(AppRoutes::GalleryDetail { id: id.clone() })
        }
        (Some(kind), Some(id)) if kind == "shoot" => {
            Some(AppRoutes::ShootDetail { id: id.clone() })
        }
        _ => None,
    };

    let media_body = if video {
        if let Some(url) = &item.media_url {
            html! {
                <div class="mark">
                    <video
                        class="w-full object-cover"
                        controls={true}
                        preload="metadata"
                        poster={item.thumbnail_url.clone()}
                    >
                        <source src={url.clone()} />
                    </video>
                </div>
            }
        } else if let Some(url) = &item.link_url {
            html! {
                <a href={url.clone()} target="_blank" rel="noopener noreferrer">
                    <LazyImage src={item.thumbnail_url.clone()} alt={title.clone()} />
                </a>
            }
        } else {
            html! { <LazyImage src={item.thumbnail_url.clone()} alt={title.clone()} /> }
        }
    } else if let Some(url) = &item.link_url {
        html! {
            <a href={url.clone()} target="_blank" rel="noopener noreferrer">
                <LazyImage src={item.thumbnail_url.clone()} alt={title.clone()} />
            </a>
        }
    } else {
        html! { <LazyImage src={item.thumbnail_url.clone()} alt={title.clone()} /> }
    };

    let print_button = if props.on_select_print.is_some() && is_printable(item) {
        let on_select = props.on_select_print.clone().unwrap();
        let item_for_click = item.clone();
        let selected = props.print_selected;
        html! {
            <button
                type="button"
                class={if selected { "btn btn-primary btn-sm" } else { "btn btn-outline btn-sm" }}
                onclick={Callback::from(move |_| on_select.emit(item_for_click.clone()))}
            >
                { if selected { "Selected for print" } else { "Select for print" } }
            </button>
        }
    } else {
        html! {}
    };

    html! {
        <article
            class={if item.featured {
                "mb-4 break-inside-avoid overflow-hidden rounded-2xl border-2 border-accent/40 bg-surface-elevated shadow-lg shadow-accent/5"
            } else {
                "mb-4 break-inside-avoid card card-flat overflow-hidden"
            }}
        >
            <div class="overflow-hidden bg-surface">
                { media_body }
            </div>
            <div class="space-y-2 px-4 py-3">
                <div class="flex flex-wrap items-center gap-2">
                    <Badge>{ html! { { source_label(&item.source) } } }</Badge>
                    if video {
                        <Badge variant={BadgeVariant::Outline}>{ html! { "Video" } }</Badge>
                    }
                    if item.featured {
                        <Badge variant={BadgeVariant::Secondary}>{ html! { "Featured" } }</Badge>
                    }
                </div>
                <p class="text-sm font-medium text-foreground">{ title }</p>
                if let Some(collection_title) = &item.collection_title {
                    if let Some(route) = collection_link {
                        <Link<AppRoutes>
                            to={route}
                            classes="text-xs text-accent hover:underline"
                        >
                            { format!("From {}", collection_title) }
                        </Link<AppRoutes>>
                    } else {
                        <p class="text-xs text-muted">{ format!("From {}", collection_title) }</p>
                    }
                }
                if let Some(date) = &item.published_at {
                    <p class="text-xs text-muted">{ date }</p>
                }
                { print_button }
            </div>
        </article>
    }
}
