use crate::{
    components::ui::LazyImage,
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
        .clone()
        .unwrap_or_else(|| String::from("https://picsum.photos/seed/cover/800/500"));

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
            classes="group block overflow-hidden rounded-2xl border border-border bg-surface-elevated transition hover:border-accent/50 hover:shadow-md hover:shadow-black/5 dark:hover:shadow-black/20"
        >
            <div class="aspect-[16/9] overflow-hidden bg-surface">
                <LazyImage
                    src={cover}
                    alt={props.summary.title.clone()}
                    class="h-full w-full object-cover transition group-hover:scale-[1.02]"
                />
            </div>
            <div class="space-y-1 p-4">
                <div class="flex items-start justify-between gap-2">
                    <h2 class="font-semibold text-foreground group-hover:text-accent">
                        { props.summary.title.clone() }
                    </h2>
                    if props.summary.has_video {
                        <span class="shrink-0 rounded bg-accent/15 px-2 py-0.5 text-xs font-medium text-accent">
                            { "Video" }
                        </span>
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
