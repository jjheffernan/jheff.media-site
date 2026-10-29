use crate::context::{ThemeContext, ThemePreference};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or(false)]
    pub inline: bool,
}

#[function_component(ThemeToggle)]
pub fn theme_toggle(props: &Props) -> Html {
    let theme_ctx = use_context::<ThemeContext>().expect("ThemeProvider required");
    let open = use_state(|| false);
    let chosen = theme_ctx.preference.effective();

    let toggle = {
        let open = open.clone();
        Callback::from(move |_| open.set(!*open))
    };

    let pick = {
        let theme_ctx = theme_ctx.clone();
        let open = open.clone();
        Callback::from(move |pref: ThemePreference| {
            theme_ctx.set_preference.emit(pref);
            open.set(false);
        })
    };

    let icon_label = match chosen {
        ThemePreference::Light => "Theme: Light",
        ThemePreference::Dark => "Theme: Dark",
        ThemePreference::Auto => "Theme",
    };

    if props.inline {
        return html! {
            <div class="menu-theme">
                <p class="menu-label">{ "Theme" }</p>
                <div class="menu-choice-row" role="group" aria-label="Theme">
                    { theme_choice("Light", ThemePreference::Light, chosen, pick.clone()) }
                    { theme_choice("Dark", ThemePreference::Dark, chosen, pick) }
                </div>
            </div>
        };
    }

    html! {
        <div class="relative">
            <button
                type="button"
                class="icon-btn"
                aria-label={icon_label}
                onclick={toggle}
            >
                { theme_icon(chosen) }
            </button>
            if *open {
                <div class="menu">
                    { theme_option("Light", ThemePreference::Light, chosen, pick.clone()) }
                    { theme_option("Dark", ThemePreference::Dark, chosen, pick) }
                </div>
            }
        </div>
    }
}

fn theme_choice(
    label: &'static str,
    value: ThemePreference,
    current: ThemePreference,
    pick: Callback<ThemePreference>,
) -> Html {
    let active = value == current;
    let class = if active {
        "menu-choice is-active"
    } else {
        "menu-choice"
    };
    html! {
        <button
            type="button"
            class={class}
            aria-pressed={active.to_string()}
            onclick={pick.reform(move |_| value)}
        >
            { label }
        </button>
    }
}

fn theme_option(
    label: &'static str,
    value: ThemePreference,
    current: ThemePreference,
    pick: Callback<ThemePreference>,
) -> Html {
    let active = value == current;
    html! {
        <button
            type="button"
            class={if active {
                "block w-full px-3 py-2 text-left text-sm font-medium text-accent"
            } else {
                "block w-full px-3 py-2 text-left text-sm text-foreground transition hover:bg-surface"
            }}
            onclick={pick.reform(move |_| value)}
        >
            { label }
            if active {
                { " ✓" }
            }
        </button>
    }
}

fn theme_icon(pref: ThemePreference) -> Html {
    match pref {
        ThemePreference::Light => html! {
            <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75">
                <circle cx="12" cy="12" r="4" />
                <path stroke-linecap="round" d="M12 2v2M12 20v2M4 12H2M22 12h-2M5 5l1.5 1.5M19.5 19.5L18 18M5 19l1.5-1.5M19.5 5L18 6.5" />
            </svg>
        },
        ThemePreference::Dark | ThemePreference::Auto => html! {
            <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75">
                <path stroke-linecap="round" d="M21 14.5A8.5 8.5 0 0110 4a8.5 8.5 0 1011 10.5z" />
            </svg>
        },
    }
}
