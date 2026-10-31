use super::content_nav::ContentNav;
use super::{profile_menu::ProfileMenu, theme_toggle::ThemeToggle};
use crate::components::ui::{NavLink, Text, TextTone};
use crate::routes::{AppRoutes, SearchQuery};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
struct SearchBoxProps {
    #[prop_or(false)]
    wide: bool,
}

#[function_component(SearchBox)]
fn search_box(props: &SearchBoxProps) -> Html {
    let navigator = use_navigator().unwrap();
    let location = use_location();
    let text = use_state(String::new);
    let from_url = location
        .and_then(|loc| loc.query::<SearchQuery>().ok())
        .map(|query| query.q)
        .unwrap_or_default();

    {
        let text = text.clone();
        use_effect_with(from_url, move |query| {
            text.set(query.clone());
            || ()
        });
    }

    let on_input = {
        let text = text.clone();
        Callback::from(move |event: InputEvent| {
            let input = event.target_unchecked_into::<web_sys::HtmlInputElement>();
            text.set(input.value());
        })
    };
    let on_submit = {
        let text = text.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let q = text.trim().to_string();
            if q.chars().count() < 2 {
                return;
            }
            let _ = navigator.push_with_query(&AppRoutes::Search, &SearchQuery { q });
        })
    };

    let class = if props.wide {
        "site-search is-wide"
    } else {
        "site-search"
    };
    html! {
        <form class={class} onsubmit={on_submit}>
            <input
                type="search"
                class="field"
                placeholder="Search"
                aria-label="Search"
                value={(*text).clone()}
                oninput={on_input}
            />
        </form>
    }
}

#[derive(Properties, Clone, PartialEq, Debug)]
pub struct Props {
    #[prop_or_default]
    pub class: String,
}

#[function_component(Header)]
pub fn header(props: &Props) -> Html {
    let menu_open = use_state(|| false);
    let current = use_route::<AppRoutes>().unwrap_or(AppRoutes::NotFound);

    {
        let menu_open = menu_open.clone();
        use_effect_with(current, move |_| {
            menu_open.set(false);
            || ()
        });
    }

    let toggle_menu = {
        let menu_open = menu_open.clone();
        Callback::from(move |_| menu_open.set(!*menu_open))
    };

    let open = *menu_open;

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

                <nav class="site-nav-desktop" aria-label="Primary">
                    <NavLink route={AppRoutes::Home} label="Home" />
                    <ContentNav />
                    <NavLink route={AppRoutes::Booking} label="Booking" />
                    <NavLink route={AppRoutes::Schedule} label="Schedule" />
                </nav>

                <div class="site-header-actions">
                    <div class="site-header-tools">
                        <SearchBox />
                        <ThemeToggle />
                        <ProfileMenu />
                    </div>
                    <button
                        type="button"
                        class="icon-btn site-nav-toggle"
                        aria-label={if open { "Close menu" } else { "Open menu" }}
                        aria-expanded={open.to_string()}
                        aria-controls="site-nav-panel"
                        onclick={toggle_menu}
                    >
                        { menu_icon(open) }
                    </button>
                </div>
            </div>

            if open {
                <nav id="site-nav-panel" class="site-nav-panel" aria-label="Primary">
                    <SearchBox wide={true} />
                    <NavLink route={AppRoutes::Home} label="Home" />
                    <ContentNav inline={true} />
                    <NavLink route={AppRoutes::Booking} label="Booking" />
                    <NavLink route={AppRoutes::Schedule} label="Schedule" />
                    <ProfileMenu inline={true} />
                </nav>
            }
        </header>
    }
}

fn menu_icon(open: bool) -> Html {
    if open {
        html! {
            <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" aria-hidden="true">
                <path stroke-linecap="round" d="M6 6l12 12M18 6L6 18" />
            </svg>
        }
    } else {
        html! {
            <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" aria-hidden="true">
                <path stroke-linecap="round" d="M4 7h16M4 12h16M4 17h16" />
            </svg>
        }
    }
}
