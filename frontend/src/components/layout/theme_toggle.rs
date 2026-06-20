use crate::context::{ThemeContext, ThemePreference};
use yew::prelude::*;

#[function_component(ThemeToggle)]
pub fn theme_toggle() -> Html {
    let theme_ctx = use_context::<ThemeContext>().expect("ThemeProvider required");
    let open = use_state(|| false);

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

    let icon_label = match theme_ctx.preference {
        ThemePreference::Auto => "Theme: Auto",
        ThemePreference::Light => "Theme: Light",
        ThemePreference::Dark => "Theme: Dark",
    };

    html! {
        <div class="relative">
            <button
                type="button"
                class="flex h-10 w-10 items-center justify-center rounded-full border border-border bg-surface-elevated text-foreground transition hover:border-accent hover:text-accent"
                aria-label={icon_label}
                onclick={toggle}
            >
                { theme_icon(theme_ctx.preference) }
            </button>
            if *open {
                <div class="absolute right-0 top-full z-50 mt-2 min-w-[9rem] rounded-xl border border-border bg-surface-elevated py-1 shadow-lg">
                    { theme_option("Auto", ThemePreference::Auto, theme_ctx.preference, pick.clone()) }
                    { theme_option("Light", ThemePreference::Light, theme_ctx.preference, pick.clone()) }
                    { theme_option("Dark", ThemePreference::Dark, theme_ctx.preference, pick.clone()) }
                </div>
            }
        </div>
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
        ThemePreference::Dark => html! {
            <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75">
                <path stroke-linecap="round" d="M21 14.5A8.5 8.5 0 0110 4a8.5 8.5 0 1011 10.5z" />
            </svg>
        },
        ThemePreference::Auto => html! {
            <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75">
                <circle cx="12" cy="12" r="9" />
                <path stroke-linecap="round" d="M12 3v18M3 12h9" fill="currentColor" opacity="0.35" />
            </svg>
        },
    }
}
