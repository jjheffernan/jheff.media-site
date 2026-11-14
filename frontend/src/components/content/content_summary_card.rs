use crate::{
    components::ui::{AspectKind, AspectRatio, Badge, LazyImage},
    model::ContentCollectionSummary,
    routes::AppRoutes,
};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub summary: ContentCollectionSummary,
    pub detail_route: AppRoutes,
}

#[function_component(ContentSummaryCard)]
pub fn content_summary_card(props: &Props) -> Html {
    let cover = props
        .summary
        .cover_url
        .as_ref()
        .filter(|url| !url.is_empty())
        .cloned();

    let meta = {
        let mut parts = vec![];
        if let Some(date) = &props.summary.date {
            parts.push(date.clone());
        }
        if let Some(location) = &props.summary.location {
            parts.push(location.clone());
        }
        if props.summary.media_count > 0 {
            parts.push(format!(
                "{} {}",
                props.summary.media_count,
                if props.summary.media_count == 1 {
                    "item"
                } else {
                    "items"
                }
            ));
        }
        if let Some(status) = &props.summary.status {
            parts.push(status.clone());
        }
        parts.join(" · ")
    };

    html! {
        <Link<AppRoutes>
            to={props.detail_route.clone()}
            classes="card card-flat overflow-hidden group"
        >
            if let Some(cover) = cover {
                <AspectRatio kind={AspectKind::Video} class="media-thumb rounded-none border-0">
                    <LazyImage
                        src={cover}
                        alt={props.summary.title.clone()}
                        class="h-full w-full object-cover transition group-hover:scale-[1.02]"
                    />
                </AspectRatio>
            }
            <div class="space-y-1 p-4">
                <div class="flex items-start justify-between gap-2">
                    <h2 class="font-semibold text-foreground group-hover:text-accent">
                        { props.summary.title.clone() }
                    </h2>
                    if props.summary.has_video {
                        <Badge>{ html! { "Video" } }</Badge>
                    }
                </div>
                if let Some(summary) = &props.summary.summary {
                    <p class="text-sm text-muted line-clamp-2">{ summary.clone() }</p>
                }
                if !meta.is_empty() {
                    <p class="text-xs text-muted">{ meta }</p>
                }
            </div>
        </Link<AppRoutes>>
    }
}
