use super::platform_badge::PlatformBadge;
use crate::components::ui::LazyImage;
use crate::model::SocialPost;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub post: SocialPost,
}

#[function_component(PostCard)]
pub fn post_card(props: &Props) -> Html {
    let post = &props.post;
    html! {
        <a
            href={post.url.clone()}
            target="_blank"
            rel="noopener noreferrer"
            class="group flex flex-col overflow-hidden rounded-xl border border-border bg-surface-elevated transition hover:border-accent/50"
        >
            if let Some(thumb) = &post.thumbnail_url {
                <div class="aspect-video overflow-hidden bg-surface">
                    <LazyImage src={thumb.clone()} alt={post.title.clone()} />
                </div>
            }
            <div class="flex flex-col gap-2 p-4">
                <PlatformBadge platform={post.platform.clone()} />
                <p class="font-medium text-foreground group-hover:text-accent">
                    { post.title.clone() }
                </p>
                if let Some(date) = &post.published_at {
                    <p class="text-xs text-muted">{ date }</p>
                }
            </div>
        </a>
    }
}
