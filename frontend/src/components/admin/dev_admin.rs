use crate::context::{AuthContext, DevModeContext, ThemeContext};
use crate::routes::AppRoutes;
use crate::services::{api_errors, auth_header};
use gloo_net::http::Request;
use serde::Deserialize;
use serde_json::json;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlTextAreaElement, MouseEvent};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ServerDiag {
    #[serde(default)]
    db_reachable: bool,
    #[serde(default)]
    feed_provider: String,
    #[serde(default)]
    rust_log: String,
    #[serde(default)]
    uptime_secs: u64,
    #[serde(default)]
    version: String,
    #[serde(default)]
    build_id: String,
}

#[derive(Clone, PartialEq)]
struct ProbeRow {
    path: String,
    ok: bool,
    detail: String,
}

#[function_component(DevAdmin)]
pub fn dev_admin() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let dev_ctx = use_context::<DevModeContext>().expect("DevModeProvider required");
    let theme_ctx = use_context::<ThemeContext>().expect("ThemeProvider required");
    let route = use_route::<AppRoutes>().unwrap_or(AppRoutes::Admin);

    let is_admin = auth_ctx
        .auth
        .as_ref()
        .is_some_and(|a| a.user.role == "admin");
    if !is_admin {
        return html! {};
    }

    let role = auth_ctx
        .auth
        .as_ref()
        .map(|a| a.user.role.clone())
        .unwrap_or_else(|| "unknown".into());
    let jwt = auth_ctx
        .auth
        .as_ref()
        .map(|a| a.jwt.clone())
        .unwrap_or_default();

    let server = use_state(ServerDiag::default);
    let probes = use_state(Vec::<ProbeRow>::new);
    let errors = use_state(api_errors::recent);
    let snapshot_el = use_node_ref();
    let note = use_state(|| None::<String>);

    let origin = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_else(|| "/".into());
    let api_base = format!("{origin}/api");
    let frontend_version = env!("CARGO_PKG_VERSION");

    let run_probes = {
        let server = server.clone();
        let probes = probes.clone();
        let errors = errors.clone();
        let note = note.clone();
        let jwt = jwt.clone();
        Callback::from(move |_: ()| {
            let server = server.clone();
            let probes = probes.clone();
            let errors = errors.clone();
            let note = note.clone();
            let jwt = jwt.clone();
            spawn_local(async move {
                note.set(None);
                let auth_val = auth_header(&jwt);
                let mut rows = Vec::new();

                // Server diagnostics (admin JWT).
                match Request::get("/api/admin/diagnostics")
                    .header("Authorization", auth_val.as_str())
                    .send()
                    .await
                {
                    Ok(resp) => {
                        let status = resp.status();
                        if resp.ok() {
                            match resp.json::<ServerDiag>().await {
                                Ok(body) => {
                                    rows.push(ProbeRow {
                                        path: "/api/admin/diagnostics".into(),
                                        ok: true,
                                        detail: format!(
                                            "db={} provider={} uptime={}s",
                                            body.db_reachable, body.feed_provider, body.uptime_secs
                                        ),
                                    });
                                    server.set(body);
                                }
                                Err(_) => {
                                    api_errors::push("/api/admin/diagnostics", status);
                                    rows.push(ProbeRow {
                                        path: "/api/admin/diagnostics".into(),
                                        ok: false,
                                        detail: "bad json".into(),
                                    });
                                }
                            }
                        } else {
                            api_errors::push("/api/admin/diagnostics", status);
                            rows.push(ProbeRow {
                                path: "/api/admin/diagnostics".into(),
                                ok: false,
                                detail: status.to_string(),
                            });
                        }
                    }
                    Err(err) => {
                        api_errors::push("/api/admin/diagnostics", 0);
                        rows.push(ProbeRow {
                            path: "/api/admin/diagnostics".into(),
                            ok: false,
                            detail: err.to_string(),
                        });
                    }
                }

                for path in ["/healthz", "/api/feed", "/api/schedule", "/metrics"] {
                    match Request::get(path).send().await {
                        Ok(resp) => {
                            let status = resp.status();
                            let ok = resp.ok();
                            if !ok {
                                api_errors::push(path, status);
                            }
                            rows.push(ProbeRow {
                                path: path.into(),
                                ok,
                                detail: status.to_string(),
                            });
                        }
                        Err(err) => {
                            api_errors::push(path, 0);
                            rows.push(ProbeRow {
                                path: path.into(),
                                ok: false,
                                detail: err.to_string(),
                            });
                        }
                    }
                }

                probes.set(rows);
                errors.set(api_errors::recent());
            });
        })
    };

    {
        let run_probes = run_probes.clone();
        let on = dev_ctx.enabled;
        use_effect_with(on, move |on| {
            if *on {
                run_probes.emit(());
            }
            || ()
        });
    }

    let refresh = {
        let run_probes = run_probes.clone();
        Callback::from(move |_: MouseEvent| run_probes.emit(()))
    };

    let toggle = {
        let set = dev_ctx.set_enabled.clone();
        let cur = dev_ctx.enabled;
        Callback::from(move |_: Event| set.emit(!cur))
    };

    let clear_errors = {
        let errors = errors.clone();
        Callback::from(move |_: MouseEvent| {
            api_errors::clear();
            errors.set(Vec::new());
        })
    };

    let select_snapshot = {
        let snapshot_el = snapshot_el.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(el) = snapshot_el.cast::<HtmlTextAreaElement>() {
                el.select();
            }
        })
    };

    let route_label = format!("{route:?}");
    let theme_label = theme_ctx.preference.as_str().to_string();
    let feed = if server.feed_provider.is_empty() {
        "—".into()
    } else {
        server.feed_provider.clone()
    };

    let snapshot = json!({
        "role": role,
        "route": route_label,
        "theme": theme_label,
        "frontendVersion": frontend_version,
        "apiBase": api_base,
        "feedProvider": feed,
        "server": {
            "dbReachable": server.db_reachable,
            "rustLog": server.rust_log,
            "uptimeSecs": server.uptime_secs,
            "version": server.version,
            "buildId": server.build_id,
        },
        "recentErrors": &*errors,
        // Never include jwt / Authorization / secrets.
    });
    let snapshot_text = serde_json::to_string_pretty(&snapshot).unwrap_or_else(|_| "{}".into());

    html! {
        <div class="space-y-4">
            <div>
                <h2 class="title-sm">{ "Dev mode" }</h2>
                <p class="muted">
                    { "Admin-only live troubleshooting. Preference stays in this browser (localStorage). Not the install DEV_MODE flag." }
                </p>
            </div>

            <section class="card space-y-3">
                <label class="row-actions" style="align-items:center;gap:0.75rem;">
                    <input type="checkbox" checked={dev_ctx.enabled} onchange={toggle} />
                    <span>{ if dev_ctx.enabled { "On — troubleshooting UI visible" } else { "Off" } }</span>
                </label>
                if let Some(msg) = &*note {
                    <p class="muted" role="status">{ msg }</p>
                }
            </section>

            if dev_ctx.enabled {
                <section class="card space-y-2">
                    <p class="label-xs">{ "Runtime" }</p>
                    <dl class="dev-kv">
                        <dt>{ "Frontend" }</dt><dd>{ frontend_version }</dd>
                        <dt>{ "API base" }</dt><dd><code>{ api_base.clone() }</code></dd>
                        <dt>{ "Feed provider" }</dt><dd>{ feed.clone() }</dd>
                        <dt>{ "DB reachable" }</dt><dd>{ server.db_reachable.to_string() }</dd>
                        <dt>{ "RUST_LOG" }</dt><dd>{ if server.rust_log.is_empty() { "—" } else { server.rust_log.as_str() } }</dd>
                        <dt>{ "Uptime" }</dt><dd>{ format!("{}s", server.uptime_secs) }</dd>
                        <dt>{ "Backend version" }</dt><dd>{ if server.version.is_empty() { "—" } else { server.version.as_str() } }</dd>
                        <dt>{ "Build id" }</dt><dd>{ if server.build_id.is_empty() { "—" } else { server.build_id.as_str() } }</dd>
                    </dl>
                </section>

                <section class="card space-y-2">
                    <div class="row-actions" style="align-items:center;justify-content:space-between;">
                        <p class="label-xs">{ "Probes" }</p>
                        <button type="button" class="btn" onclick={refresh}>{ "Probe now" }</button>
                    </div>
                    <ul class="dev-list">
                        { for probes.iter().map(|row| html! {
                            <li class={if row.ok { "dev-ok" } else { "dev-bad" }}>
                                <code>{ &row.path }</code>
                                { " — " }
                                { &row.detail }
                            </li>
                        }) }
                    </ul>
                    <p class="muted label-xs">{ "Quick links (same origin):" }</p>
                    <p class="row-actions">
                        <a class="btn" href="/healthz" target="_blank" rel="noopener">{ "/healthz" }</a>
                        <a class="btn" href="/metrics" target="_blank" rel="noopener">{ "/metrics" }</a>
                        <a class="btn" href="/api/feed" target="_blank" rel="noopener">{ "/api/feed" }</a>
                        <a class="btn" href="/api/schedule" target="_blank" rel="noopener">{ "/api/schedule" }</a>
                    </p>
                </section>

                <section class="card space-y-2">
                    <div class="row-actions" style="align-items:center;justify-content:space-between;">
                        <p class="label-xs">{ "Recent failed fetches" }</p>
                        <button type="button" class="btn" onclick={clear_errors}>{ "Clear" }</button>
                    </div>
                    if errors.is_empty() {
                        <p class="muted">{ "None yet (probes and tracked failures land here)." }</p>
                    } else {
                        <ul class="dev-list">
                            { for errors.iter().rev().map(|e| html! {
                                <li class="dev-bad">
                                    <code>{ &e.path }</code>
                                    { format!(" — {}", e.status) }
                                </li>
                            }) }
                        </ul>
                    }
                </section>

                <section class="card space-y-2">
                    <div class="row-actions" style="align-items:center;justify-content:space-between;">
                        <p class="label-xs">{ "Diagnostics snapshot (masked)" }</p>
                        <button type="button" class="btn" onclick={select_snapshot}>{ "Select all" }</button>
                    </div>
                    <textarea
                        ref={snapshot_el}
                        class="field"
                        readonly=true
                        rows="12"
                        value={snapshot_text}
                        aria-label="Diagnostics snapshot JSON"
                    />
                    <p class="muted label-xs">{ "No JWT, passwords, or API keys. Safe to paste into a ticket." }</p>
                </section>
            }
        </div>
    }
}
