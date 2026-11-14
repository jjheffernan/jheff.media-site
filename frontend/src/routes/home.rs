use crate::components::{
    ui::{Badge, Separator},
    Hero, InstagramFeaturedCard, LinkCard, MediaFeedScroll, Stack, StackGap,
};
use crate::routes::AppRoutes;
use yew::prelude::*;

/// Public homepage — media-first hub for @jheffmedia.
pub struct Home {}

impl Component for Home {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <Stack gap={StackGap::Sm}>
                <Hero />

                <div class="home-intro">
                    <div class="stack-gap-xs">
                        <Badge>{ html! { "Feed" } }</Badge>
                        <p class="muted">{ "Latest stills and social from the road." }</p>
                    </div>
                </div>

                <MediaFeedScroll limit={24} />

                <Separator />

                <InstagramFeaturedCard />

                <div class="home-actions">
                    <LinkCard route={AppRoutes::Content} title="Content hub">
                        { "Social posts, shoots, and galleries — everything @jheffmedia in one view." }
                    </LinkCard>
                    <LinkCard route={AppRoutes::Booking} title="Booking">
                        { "Request track instruction, media shoots, or race engineering sessions." }
                    </LinkCard>
                    <LinkCard route={AppRoutes::Contact} title="Contact">
                        { "General questions and collaborations." }
                    </LinkCard>
                </div>
            </Stack>
        }
    }
}
