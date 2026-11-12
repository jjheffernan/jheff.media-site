use gloo_storage::{LocalStorage, Storage};
use yew::prelude::*;

const STORAGE_KEY: &str = "admin-dev-mode";

#[derive(Clone, PartialEq)]
pub struct DevModeContext {
    pub enabled: bool,
    pub set_enabled: Callback<bool>,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub children: Children,
}

#[function_component(DevModeProvider)]
pub fn dev_mode_provider(props: &Props) -> Html {
    let enabled = use_state(load);
    let set_enabled = {
        let enabled = enabled.clone();
        Callback::from(move |on: bool| {
            save(on);
            enabled.set(on);
        })
    };
    let ctx = DevModeContext {
        enabled: *enabled,
        set_enabled,
    };
    html! {
        <ContextProvider<DevModeContext> context={ctx}>
            { for props.children.iter() }
        </ContextProvider<DevModeContext>>
    }
}

fn load() -> bool {
    LocalStorage::get::<String>(STORAGE_KEY)
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn save(on: bool) {
    LocalStorage::set(STORAGE_KEY, if on { "1" } else { "0" }).ok();
}
