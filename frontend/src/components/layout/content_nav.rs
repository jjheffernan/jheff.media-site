use crate::routes::AppRoutes;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Panel {
    Closed,
    Open,
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
pub fn content_nav() -> Html {
    let panel = use_state(|| Panel::Closed);
    let current = use_route::<AppRoutes>().unwrap_or(AppRoutes::NotFound);
    let active = is_content_route(&current);

    {
        let panel = panel.clone();
        let current = current.clone();
        use_effect_with(current, move |_| {
            panel.set(Panel::Closed);
            || ()
        });
    }

    let close_panel = {
        let panel = panel.clone();
        Callback::from(move |_| panel.set(Panel::Closed))
    };

    let navigator = use_navigator().unwrap();

    let open_panel = {
        let panel = panel.clone();
        Callback::from(move |_| panel.set(Panel::Open))
    };

    let go_content_hub = {
        let navigator = navigator.clone();
        let panel = panel.clone();
        Callback::from(move |_| {
            navigator.push(&AppRoutes::Content);
            panel.set(Panel::Closed);
        })
    };

    let go_galleries = {
        let navigator = navigator.clone();
        let panel = panel.clone();
        Callback::from(move |_| {
            navigator.push(&AppRoutes::Galleries);
            panel.set(Panel::Closed);
        })
    };

    let go_shoots = {
        let navigator = navigator.clone();
        let panel = panel.clone();
        Callback::from(move |_| {
            navigator.push(&AppRoutes::Shoots);
            panel.set(Panel::Closed);
        })
    };

    let go_social = {
        let navigator = navigator.clone();
        let panel = panel.clone();
        Callback::from(move |_| {
            navigator.push(&AppRoutes::Social);
            panel.set(Panel::Closed);
        })
    };

    let trigger_class = if active {
        "inline-flex items-center gap-1 rounded-md bg-surface-elevated px-3 py-2 text-sm font-medium text-foreground"
    } else {
        "inline-flex items-center gap-1 rounded-md px-3 py-2 text-sm font-medium text-muted transition hover:bg-surface-elevated hover:text-foreground"
    };

    let item_class = "block w-full px-3 py-2 text-left text-sm text-foreground transition hover:bg-surface";

    html! {
        <div
            class="relative"
            onmouseleave={close_panel.clone()}
        >
            <button
                type="button"
                class={trigger_class}
                aria-label="Content menu — double-click for hub"
                aria-expanded={(*panel == Panel::Open).to_string()}
                aria-haspopup="true"
                onclick={open_panel}
                ondblclick={go_content_hub}
            >
                { "Content" }
                <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 10.94l3.71-3.71a.75.75 0 111.06 1.06l-4.24 4.25a.75.75 0 01-1.06 0L5.21 8.27a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
                </svg>
            </button>

            if *panel == Panel::Open {
                <div class="absolute left-0 top-full z-50 mt-1 min-w-[11rem] rounded-xl border border-border bg-surface-elevated py-1 shadow-lg">
                    <button type="button" class={item_class} onclick={go_galleries}>
                        { "Galleries" }
                    </button>
                    <button type="button" class={item_class} onclick={go_shoots}>
                        { "Shoots" }
                    </button>
                    <button type="button" class={item_class} onclick={go_social}>
                        { "Social" }
                    </button>
                </div>
            }
        </div>
    }
}
