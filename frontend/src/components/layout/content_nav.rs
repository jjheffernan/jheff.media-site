use crate::routes::AppRoutes;
use js_sys::Date;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Panel {
    Closed,
    Open,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or(false)]
    pub inline: bool,
}

fn is_content_route(route: &AppRoutes) -> bool {
    matches!(
        route,
        AppRoutes::Content
            | AppRoutes::Galleries
            | AppRoutes::GalleryDetail { .. }
            | AppRoutes::Shoots
            | AppRoutes::ShootDetail { .. }
            | AppRoutes::Social
    )
}

#[function_component(ContentNav)]
pub fn content_nav(props: &Props) -> Html {
    let panel = use_state(|| Panel::Closed);
    let last_tap = use_state(|| 0.0);
    let current = use_route::<AppRoutes>().unwrap_or(AppRoutes::NotFound);
    let active = is_content_route(&current);
    let open = *panel == Panel::Open;
    let navigator = use_navigator().unwrap();
    let menu_id = if props.inline {
        "content-nav-menu-mobile"
    } else {
        "content-nav-menu"
    };

    {
        let panel = panel.clone();
        let current = current.clone();
        use_effect_with(current, move |_| {
            panel.set(Panel::Closed);
            || ()
        });
    }

    let open_hub = {
        let navigator = navigator.clone();
        let panel = panel.clone();
        Callback::from(move |_: MouseEvent| {
            panel.set(Panel::Closed);
            navigator.push(&AppRoutes::Content);
        })
    };

    let on_click = {
        let panel = panel.clone();
        let last_tap = last_tap.clone();
        let navigator = navigator.clone();
        Callback::from(move |_: MouseEvent| {
            let now = Date::now();
            if now - *last_tap < 500.0 {
                last_tap.set(0.0);
                panel.set(Panel::Closed);
                navigator.push(&AppRoutes::Content);
                return;
            }
            last_tap.set(now);
            panel.set(if *panel == Panel::Open {
                Panel::Closed
            } else {
                Panel::Open
            });
        })
    };

    let close = {
        let panel = panel.clone();
        Callback::from(move |_| panel.set(Panel::Closed))
    };

    let root_class = if props.inline {
        "content-nav is-inline"
    } else {
        "content-nav"
    };
    let trigger_class = if active {
        "content-nav-trigger is-active"
    } else {
        "content-nav-trigger"
    };

    html! {
        <div class={root_class}>
            <button
                type="button"
                class={trigger_class}
                aria-label="Content menu. Double-click to open the hub."
                aria-expanded={open.to_string()}
                aria-haspopup="true"
                aria-controls={menu_id}
                onclick={on_click}
                ondblclick={open_hub}
            >
                { "Content" }
                <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                    <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 10.94l3.71-3.71a.75.75 0 111.06 1.06l-4.24 4.25a.75.75 0 01-1.06 0L5.21 8.27a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
                </svg>
            </button>
            if open {
                if !props.inline {
                    <button
                        type="button"
                        class="content-nav-backdrop"
                        aria-label="Close content menu"
                        onclick={close}
                    />
                }
                <div id={menu_id} class="content-nav-menu">
                    <Link<AppRoutes> to={AppRoutes::Galleries} classes="content-nav-item">
                        { "Galleries" }
                    </Link<AppRoutes>>
                    <Link<AppRoutes> to={AppRoutes::Shoots} classes="content-nav-item">
                        { "Shoots" }
                    </Link<AppRoutes>>
                    <Link<AppRoutes> to={AppRoutes::Social} classes="content-nav-item">
                        { "Social" }
                    </Link<AppRoutes>>
                </div>
            }
        </div>
    }
}
