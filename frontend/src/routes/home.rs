use crate::components::{Hero, Grid, LinkCard, PhotoFeed, Stack, Text, TextTone};
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
            <Stack>
                <Hero />

                <PhotoFeed title="Recent media" limit={12} />

                <Grid cols_sm={1} cols_lg={3}>
                    <LinkCard route={AppRoutes::Content} title="Content hub">
                        { "Social posts, shoots, and galleries — everything @jheffmedia in one view." }
                    </LinkCard>
                    <LinkCard route={AppRoutes::Booking} title="Booking">
                        { "Request track instruction, media shoots, or race engineering sessions." }
                    </LinkCard>
                    <LinkCard route={AppRoutes::Contact} title="Contact">
                        <Text tone={TextTone::Muted}>
                            { "Questions, collaborations, and booking inquiries." }
                        </Text>
                    </LinkCard>
                </Grid>
            </Stack>
        }
    }
}
