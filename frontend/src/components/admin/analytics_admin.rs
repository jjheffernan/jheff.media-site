use crate::{context::AuthContext, services::auth_header};
use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, Deserialize, PartialEq)]
struct Bucket {
    day: String,
    n: i64,
}

#[derive(Clone, Deserialize, PartialEq)]
struct PathCount {
    path: String,
    n: i64,
}

#[derive(Clone, Deserialize, PartialEq)]
struct Analytics {
    days: Vec<Bucket>,
    paths: Vec<PathCount>,
}

fn share(n: i64, total: i64) -> i64 {
    if total <= 0 { 0 } else { (n * 100 / total).clamp(0, 100) }
}

fn day_short(day: &str) -> String {
    day.get(5..).unwrap_or(day).to_string()
}

#[function_component(AnalyticsAdmin)]
pub fn analytics_admin() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let data = use_state(|| None::<Analytics>);
    let note = use_state(|| None::<String>);
    let jwt = auth_ctx
        .auth
        .as_ref()
        .filter(|auth| auth.user.role == "admin")
        .map(|auth| auth.jwt.clone())
        .unwrap_or_default();

    {
        let data = data.clone();
        let note = note.clone();
        use_effect_with(jwt.clone(), move |jwt| {
            if !jwt.is_empty() {
                let jwt = jwt.clone();
                spawn_local(async move {
                    let auth_val = auth_header(&jwt);
                    match Request::get("/api/admin/analytics")
                        .header("Authorization", auth_val.as_str())
                        .send()
                        .await
                    {
                        Ok(resp) if resp.ok() => match resp.json::<Analytics>().await {
                            Ok(body) => data.set(Some(body)),
                            Err(_) => note.set(Some("Could not read analytics.".into())),
                        },
                        Ok(_) => note.set(Some("Could not load analytics.".into())),
                        Err(err) => note.set(Some(err.to_string())),
                    }
                });
            }
            || ()
        });
    }

    let body = match &*data {
        None => html! { <p class="muted">{ "Loading page views…" }</p> },
        Some(report) if report.days.is_empty() && report.paths.is_empty() => {
            html! {
                <section class="card">
                    <p class="muted">{ "No page views yet." }</p>
                </section>
            }
        }
        Some(report) => {
            let total: i64 = report.days.iter().map(|row| row.n).sum();
            let day_max = report.days.iter().map(|row| row.n).max().unwrap_or(1);
            let latest = report.days.last();
            let top = report.paths.first();
            let latest_n = latest.map(|row| row.n.to_string()).unwrap_or_else(|| "0".into());
            let latest_day = latest.map(|row| row.day.clone()).unwrap_or_default();
            let top_path = top.map(|row| row.path.clone()).unwrap_or_else(|| "—".into());
            let top_n = top.map(|row| row.n.to_string()).unwrap_or_else(|| "0".into());
            html! {
                <div class="space-y-4">
                    <div>
                        <h2 class="title-sm">{ "Last 14 days" }</h2>
                        <p class="muted">{ "Page views by path and day." }</p>
                    </div>
                    <div class="stat-grid">
                        <section class="card space-y-1">
                            <p class="label-xs">{ "Views" }</p>
                            <p class="stat-n">{ total.to_string() }</p>
                        </section>
                        <section class="card space-y-1">
                            <p class="label-xs">{ "Latest day" }</p>
                            <p class="stat-n">{ latest_n }</p>
                            <p class="muted">{ latest_day }</p>
                        </section>
                        <section class="card space-y-1">
                            <p class="label-xs">{ "Top path" }</p>
                            <p class="stat-n">{ top_n }</p>
                            <p class="muted">{ top_path }</p>
                        </section>
                    </div>
                    <div class="dash-lower">
                        <section class="card space-y-3">
                            <h3 class="title-sm">{ "By day" }</h3>
                            <div class="day-chart" role="img" aria-label="Page views by day">
                                { for report.days.iter().map(|row| {
                                    let pct = share(row.n, day_max).max(8);
                                    html! {
                                        <div class="day-col" key={row.day.clone()} title={format!("{} · {}", row.day, row.n)}>
                                            <span class="day-bar" style={format!("height: calc({pct} * 0.065rem)")}></span>
                                            <span class="day-label">{ day_short(&row.day) }</span>
                                        </div>
                                    }
                                }) }
                            </div>
                        </section>
                        <section class="card space-y-3">
                            <h3 class="title-sm">{ "Paths" }</h3>
                            { for report.paths.iter().map(|row| {
                                let pct = share(row.n, total);
                                let bar = pct.max(4);
                                html! {
                                    <div class="bar-row" key={row.path.clone()}>
                                        <span class="muted" title={row.path.clone()}>{ row.path.clone() }</span>
                                        <span class="bar-track"><span class="bar" style={format!("width: {bar}%")}></span></span>
                                        <span class="muted">{ format!("{pct}%") }</span>
                                    </div>
                                }
                            }) }
                        </section>
                    </div>
                </div>
            }
        }
    };

    html! {
        <>
            if let Some(msg) = &*note {
                <p class="text-sm text-accent">{ msg }</p>
            }
            { body }
        </>
    }
}
