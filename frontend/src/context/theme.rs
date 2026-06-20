use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::Window;
use yew::prelude::*;

const STORAGE_KEY: &str = "theme-preference";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemePreference {
    Auto,
    Light,
    Dark,
}

impl ThemePreference {
    pub fn from_str(s: &str) -> Self {
        match s {
            "light" => ThemePreference::Light,
            "dark" => ThemePreference::Dark,
            _ => ThemePreference::Auto,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ThemePreference::Auto => "auto",
            ThemePreference::Light => "light",
            ThemePreference::Dark => "dark",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ThemeContext {
    pub preference: ThemePreference,
    pub set_preference: Callback<ThemePreference>,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub children: Children,
}

pub enum Msg {
    SetPreference(ThemePreference),
    SystemChanged,
}

pub struct ThemeProvider {
    preference: ThemePreference,
}

fn window() -> Option<Window> {
    web_sys::window()
}

fn system_prefers_dark() -> bool {
    window()
        .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok())
        .flatten()
        .map(|mq| mq.matches())
        .unwrap_or(false)
}

fn resolved_dark(preference: ThemePreference) -> bool {
    match preference {
        ThemePreference::Auto => system_prefers_dark(),
        ThemePreference::Light => false,
        ThemePreference::Dark => true,
    }
}

fn apply_theme(preference: ThemePreference) {
    let dark = resolved_dark(preference);
    if let Some(doc) = window().and_then(|w| w.document()) {
        if let Some(el) = doc.document_element() {
            if let Ok(html) = el.dyn_into::<web_sys::HtmlElement>() {
                if dark {
                    html.class_list().add_1("dark").ok();
                } else {
                    html.class_list().remove_1("dark").ok();
                }
            }
        }
    }
}

fn restore_preference() -> ThemePreference {
    LocalStorage::get(STORAGE_KEY)
        .ok()
        .map(|s: String| ThemePreference::from_str(&s))
        .unwrap_or(ThemePreference::Auto)
}

fn store_preference(preference: ThemePreference) {
    LocalStorage::set(STORAGE_KEY, preference.as_str()).ok();
}

impl Component for ThemeProvider {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let preference = restore_preference();
        apply_theme(preference);

        if preference == ThemePreference::Auto {
            let link = ctx.link().clone();
            if let Some(mq) = window().and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok()).flatten() {
                let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                    link.send_message(Msg::SystemChanged);
                }) as Box<dyn FnMut(web_sys::Event)>);
                mq.set_onchange(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
            }
        }

        Self { preference }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetPreference(pref) => {
                self.preference = pref;
                store_preference(pref);
                apply_theme(pref);
            }
            Msg::SystemChanged => {
                if self.preference == ThemePreference::Auto {
                    apply_theme(ThemePreference::Auto);
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let set_preference = ctx.link().callback(Msg::SetPreference);
        let theme_ctx = ThemeContext {
            preference: self.preference,
            set_preference,
        };
        html! {
            <ContextProvider<ThemeContext> context={theme_ctx}>
                { for ctx.props().children.iter() }
            </ContextProvider<ThemeContext>>
        }
    }
}
