use super::content_nav::ContentNav;
use crate::components::ui::{NavLink, Text, TextTone};
use crate::routes::AppRoutes;
use super::{profile_menu::ProfileMenu, theme_toggle::ThemeToggle};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, Clone, PartialEq, Debug)]
pub struct Props {
    #[prop_or_default]
    pub class: String,
}

#[function_component(Header)]
pub fn header(props: &Props) -> Html {
    html! {
        <header class={format!(
            "site-header {}",
            props.class
        )}>
            <div class="site-header-bar">
                <Link<AppRoutes>
                    to={AppRoutes::Home}
                    classes="brand"
                >
                    <p class="kicker">
                        { "jheff.media" }
                    </p>
                    <Text tone={TextTone::Muted} class="mt-0.5 hidden sm:block">
                        { "@jheffmedia · automotive media" }
                    </Text>
                </Link<AppRoutes>>

                <nav class="flex flex-wrap items-center gap-1">
                    <NavLink route={crate::routes::AppRoutes::Home} label="Home" />
                    <ContentNav />
                    <NavLink route={crate::routes::AppRoutes::Booking} label="Booking" />
                    <NavLink route={crate::routes::AppRoutes::Schedule} label="Schedule" />
                </nav>

                <div class="flex shrink-0 items-center gap-2">
                    <ThemeToggle />
                    <ProfileMenu />
                </div>
            </div>
        </header>
    }
}
