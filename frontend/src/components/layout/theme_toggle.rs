use crate::context::{ThemeContext, ThemePreference};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or(false)]
    pub inline: bool,
}

fn cycle(pref: ThemePreference) -> ThemePreference {
    match pref {
        ThemePreference::Light => ThemePreference::Dark,
        ThemePreference::Dark => ThemePreference::Auto,
        ThemePreference::Auto => ThemePreference::Light,
    }
}

#[function_component(ThemeToggle)]
pub fn theme_toggle(props: &Props) -> Html {
    let theme_ctx = use_context::<ThemeContext>().expect("ThemeProvider required");
    let pref = theme_ctx.preference;

    let cycle_click = {
        let theme_ctx = theme_ctx.clone();
        Callback::from(move |_| {
            theme_ctx.set_preference.emit(cycle(theme_ctx.preference));
        })
    };

    let pick = {
        let theme_ctx = theme_ctx.clone();
        Callback::from(move |next: ThemePreference| {
            theme_ctx.set_preference.emit(next);
        })
    };

    let icon_label = match pref {
        ThemePreference::Light => "Theme: Light (press for Dark)",
        ThemePreference::Dark => "Theme: Dark (press for System)",
        ThemePreference::Auto => "Theme: System (press for Light)",
    };

    if props.inline {
        return html! {
            <div class="menu-theme">
                <p class="menu-label">{ "Theme" }</p>
                <div class="menu-choice-row is-three" role="group" aria-label="Theme">
                    { theme_choice("Light", ThemePreference::Light, pref, pick.clone()) }
                    { theme_choice("Dark", ThemePreference::Dark, pref, pick.clone()) }
                    { theme_choice("System", ThemePreference::Auto, pref, pick) }
                </div>
            </div>
        };
    }

    html! {
        <button
            type="button"
            class="icon-btn"
            aria-label={icon_label}
            title={icon_label}
            onclick={cycle_click}
        >
            { theme_icon(pref) }
        </button>
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

fn theme_icon(pref: ThemePreference) -> Html {
    match pref {
        ThemePreference::Light => html! {
            <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" aria-hidden="true">
                <circle cx="12" cy="12" r="4" />
                <path stroke-linecap="round" d="M12 2v2M12 20v2M4 12H2M22 12h-2M5 5l1.5 1.5M19.5 19.5L18 18M5 19l1.5-1.5M19.5 5L18 6.5" />
            </svg>
        },
        ThemePreference::Dark => html! {
            <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" aria-hidden="true">
                <path stroke-linecap="round" d="M21 14.5A8.5 8.5 0 0110 4a8.5 8.5 0 1011 10.5z" />
            </svg>
        },
        ThemePreference::Auto => html! {
            <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" aria-hidden="true">
                <rect x="3" y="5" width="18" height="14" rx="2" />
                <path stroke-linecap="round" d="M8 19h8M12 5v-1" />
                <path stroke-linecap="round" d="M12 9v6" />
                <circle cx="12" cy="12" r="2.25" />
            </svg>
        },
    }
}
