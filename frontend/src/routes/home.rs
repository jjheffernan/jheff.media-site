use crate::components::{Hero, Grid, InstagramFeaturedCard, LinkCard, MediaFeedScroll, Stack};
use crate::routes::AppRoutes;
use yew::prelude::*;
use yew_router::prelude::*;

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
            <Stack>
                <MediaFeedScroll title="Feed" limit={24} />

                <Link<AppRoutes> to={AppRoutes::Content} classes="btn btn-primary">
                    { "Content hub" }
                </Link<AppRoutes>>

                <Hero />

                <InstagramFeaturedCard />

                <Grid cols_sm={1} cols_lg={3}>
                    <LinkCard route={AppRoutes::Content} title="Content hub">
                        { "Social posts, shoots, and galleries — everything @jheffmedia in one view." }
                    </LinkCard>
                    <LinkCard route={AppRoutes::Booking} title="Booking">
                        { "Request track instruction, media shoots, or race engineering sessions." }
                    </LinkCard>
                    <LinkCard route={AppRoutes::Contact} title="Contact">
                        { "General questions and collaborations." }
                    </LinkCard>
                </Grid>
            </Stack>
        }
    }
}
