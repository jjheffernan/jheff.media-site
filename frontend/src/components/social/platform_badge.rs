use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub platform: String,
}

#[function_component(PlatformBadge)]
pub fn platform_badge(props: &Props) -> Html {
    let (label, color) = match props.platform.to_lowercase().as_str() {
        "instagram" => ("Instagram", "bg-gradient-to-r from-purple-500 to-pink-500 text-white"),
        "youtube" => ("YouTube", "bg-red-600 text-white"),
        "twitch" => ("Twitch", "bg-purple-700 text-white"),
        "tiktok" => ("TikTok", "bg-zinc-900 text-white dark:bg-zinc-100 dark:text-zinc-900"),
        "facebook" => ("Facebook", "bg-blue-600 text-white"),
        other => (props.platform.as_str(), "bg-surface-elevated text-foreground border border-border"),
    };

    html! {
        <span class={format!("inline-flex rounded-full px-2.5 py-0.5 text-xs font-semibold {}", color)}>
            { label }
        </span>
    }
}
