use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    /// 0.0 – 1.0
    pub value: f32,
    #[prop_or_default]
    pub class: String,
}

#[function_component(Progress)]
pub fn progress(props: &Props) -> Html {
    let pct = (props.value.clamp(0.0, 1.0) * 100.0).round();
    html! {
        <div class={format!("progress {}", props.class)} role="progressbar" aria-valuenow={pct.to_string()} aria-valuemin="0" aria-valuemax="100">
            <span style={format!("width: {pct}%")}></span>
        </div>
    }
}
