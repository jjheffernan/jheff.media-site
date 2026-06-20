use crate::{
    components::ui::LazyImage,
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
                <video
                    class="w-full object-cover"
                    controls={true}
                    preload="metadata"
                    poster={item.thumbnail_url.clone()}
                >
                    <source src={url.clone()} />
                </video>
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
                class={if selected {
                    "rounded-md bg-accent px-2 py-1 text-xs font-medium text-zinc-950"
                } else {
                    "rounded-md border border-border px-2 py-1 text-xs font-medium text-foreground transition hover:border-accent hover:text-accent"
                }}
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
                "mb-4 break-inside-avoid overflow-hidden rounded-2xl border border-border bg-surface-elevated"
            }}
        >
            <div class="overflow-hidden bg-surface">
                { media_body }
            </div>
            <div class="space-y-2 px-4 py-3">
                <div class="flex flex-wrap items-center gap-2">
                    <span class="rounded bg-accent/15 px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-accent">
                        { source_label(&item.source) }
                    </span>
                    if video {
                        <span class="rounded bg-surface px-1.5 py-0.5 text-[10px] font-medium text-muted">
                            { "Video" }
                        </span>
                    }
                    if item.featured {
                        <span class="rounded bg-accent px-1.5 py-0.5 text-[10px] font-semibold text-zinc-950">
                            { "Featured" }
                        </span>
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
