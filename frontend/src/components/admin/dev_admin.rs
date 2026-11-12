use crate::context::{AuthContext, DevModeContext, ThemeContext};
use crate::routes::AppRoutes;
use crate::services::{api_errors, auth_header};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlTextAreaElement, MouseEvent};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DeployStatus {
    #[serde(default)]
    build_id: String,
    #[serde(default)]
    channel_hint: String,
    #[serde(default)]
    cli_available: bool,
    #[serde(default)]
    control_dir: bool,
    #[serde(default)]
    available: bool,
    #[serde(default)]
    autoupdate_enabled: Option<bool>,
    #[serde(default)]
    autoupdate_channel: Option<String>,
    #[serde(default)]
    active_job_id: Option<String>,
    #[serde(default)]
    message: String,
}

#[derive(Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DeployJob {
    #[serde(default)]
    id: String,
    #[serde(default)]
    channel: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    log_tail: String,
    #[serde(default)]
    exit_code: Option<i32>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeployStartResp {
    job_id: String,
}

#[derive(Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DbTestResult {
    #[serde(default)]
    ok: bool,
    #[serde(default)]
    detail: String,
}

#[derive(Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DbTests {
    #[serde(default)]
    ping: DbTestResult,
    #[serde(default)]
    list_collections: DbTestResult,
    #[serde(default)]
    users_readable: DbTestResult,
}

#[derive(Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ProdWarning {
    #[serde(default)]
    code: String,
    #[serde(default)]
    message: String,
}

#[derive(Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ServerDiag {
    #[serde(default)]
    db_reachable: bool,
    #[serde(default)]
    db_tests: DbTests,
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
    #[serde(default)]
    compress_enabled: bool,
    #[serde(default)]
    warnings: Vec<ProdWarning>,
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
    let deploy = use_state(DeployStatus::default);
    let deploy_job = use_state(|| None::<DeployJob>);
    let deploy_note = use_state(|| None::<String>);
    let deploy_busy = use_state(|| false);

    let origin = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_else(|| "/".into());
    let api_base = format!("{origin}/api");
    let frontend_version = env!("CARGO_PKG_VERSION");

    let refresh_deploy = {
        let deploy = deploy.clone();
        let deploy_job = deploy_job.clone();
        let deploy_busy = deploy_busy.clone();
        let jwt = jwt.clone();
        Callback::from(move |_: ()| {
            let deploy = deploy.clone();
            let deploy_job = deploy_job.clone();
            let deploy_busy = deploy_busy.clone();
            let jwt = jwt.clone();
            spawn_local(async move {
                let auth_val = auth_header(&jwt);
                if let Ok(resp) = Request::get("/api/admin/deploy/status")
                    .header("Authorization", auth_val.as_str())
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(body) = resp.json::<DeployStatus>().await {
                            if let Some(id) = body.active_job_id.clone() {
                                deploy_busy.set(true);
                                if let Ok(jresp) =
                                    Request::get(&format!("/api/admin/deploy/jobs/{id}"))
                                        .header("Authorization", auth_val.as_str())
                                        .send()
                                        .await
                                {
                                    if jresp.ok() {
                                        if let Ok(job) = jresp.json::<DeployJob>().await {
                                            let running =
                                                job.state == "running" || job.state == "queued";
                                            deploy_busy.set(running);
                                            deploy_job.set(Some(job));
                                        }
                                    }
                                }
                            } else {
                                deploy_busy.set(false);
                            }
                            deploy.set(body);
                        }
                    }
                }
            });
        })
    };

    {
        let refresh_deploy = refresh_deploy.clone();
        use_effect_with((), move |_| {
            refresh_deploy.emit(());
            || ()
        });
    }

    // Poll while a deploy job is active (site may briefly drop during rebuild).
    {
        let deploy_job = deploy_job.clone();
        let deploy_busy = deploy_busy.clone();
        let jwt = jwt.clone();
        let refresh_deploy = refresh_deploy.clone();
        use_effect_with(
            (
                (*deploy_busy).clone(),
                deploy_job.as_ref().map(|j| j.id.clone()),
            ),
            move |(busy, job_id)| {
                if !*busy {
                    return Box::new(|| ()) as Box<dyn FnOnce()>;
                }
                let Some(id) = job_id.clone() else {
                    return Box::new(|| ()) as Box<dyn FnOnce()>;
                };
                let deploy_job = deploy_job.clone();
                let deploy_busy = deploy_busy.clone();
                let jwt = jwt.clone();
                let refresh_deploy = refresh_deploy.clone();
                let cancelled = std::rc::Rc::new(std::cell::Cell::new(false));
                let cancelled_flag = cancelled.clone();
                spawn_local(async move {
                    while !cancelled_flag.get() {
                        sleep_ms(2500).await;
                        if cancelled_flag.get() {
                            break;
                        }
                        let auth_val = auth_header(&jwt);
                        match Request::get(&format!("/api/admin/deploy/jobs/{id}"))
                            .header("Authorization", auth_val.as_str())
                            .send()
                            .await
                        {
                            Ok(resp) if resp.ok() => {
                                if let Ok(job) = resp.json::<DeployJob>().await {
                                    let running =
                                        job.state == "running" || job.state == "queued";
                                    deploy_busy.set(running);
                                    deploy_job.set(Some(job));
                                    if !running {
                                        refresh_deploy.emit(());
                                        break;
                                    }
                                }
                            }
                            _ => {
                                // Rebuild may restart the container — keep waiting.
                            }
                        }
                    }
                });
                Box::new(move || cancelled.set(true)) as Box<dyn FnOnce()>
            },
        );
    }

    let start_deploy = {
        let deploy_busy = deploy_busy.clone();
        let deploy_job = deploy_job.clone();
        let deploy_note = deploy_note.clone();
        let jwt = jwt.clone();
        Callback::from(move |channel: &'static str| {
            let msg = if channel == "testing" {
                "Update from testing branch? Rebuilds containers; site may briefly restart."
            } else {
                "Update production (main)? Rebuilds containers; site may briefly restart."
            };
            let ok = web_sys::window()
                .and_then(|w| w.confirm_with_message(msg).ok())
                .unwrap_or(false);
            if !ok {
                return;
            }
            let deploy_busy = deploy_busy.clone();
            let deploy_job = deploy_job.clone();
            let deploy_note = deploy_note.clone();
            let jwt = jwt.clone();
            spawn_local(async move {
                deploy_note.set(None);
                deploy_busy.set(true);
                let auth_val = auth_header(&jwt);
                let body = json!({ "channel": channel });
                match Request::post("/api/admin/deploy/update")
                    .header("Authorization", auth_val.as_str())
                    .header("Content-Type", "application/json")
                    .body(body.to_string())
                {
                    Ok(req) => match req.send().await {
                        Ok(resp) => {
                            let status = resp.status();
                            if status == 202 || resp.ok() {
                                if let Ok(start) = resp.json::<DeployStartResp>().await {
                                    deploy_job.set(Some(DeployJob {
                                        id: start.job_id.clone(),
                                        channel: channel.into(),
                                        state: "queued".into(),
                                        log_tail: String::new(),
                                        exit_code: None,
                                    }));
                                    deploy_note.set(Some(format!(
                                        "Started job {} ({channel})",
                                        start.job_id
                                    )));
                                }
                            } else {
                                deploy_busy.set(false);
                                let text = resp.text().await.unwrap_or_default();
                                deploy_note.set(Some(format!("Deploy failed ({status}): {text}")));
                            }
                        }
                        Err(err) => {
                            deploy_busy.set(false);
                            deploy_note.set(Some(format!("Deploy request failed: {err}")));
                        }
                    },
                    Err(err) => {
                        deploy_busy.set(false);
                        deploy_note.set(Some(format!("Deploy request build failed: {err}")));
                    }
                }
            });
        })
    };

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
                                            "db={} ping={} provider={} uptime={}s",
                                            body.db_reachable,
                                            body.db_tests.ping.ok,
                                            body.feed_provider,
                                            body.uptime_secs
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
                    match Request::get(path)
                        .header("Accept-Encoding", "gzip, deflate, br")
                        .send()
                        .await
                    {
                        Ok(resp) => {
                            let status = resp.status();
                            let ok = resp.ok();
                            let encoding = resp
                                .headers()
                                .get("content-encoding")
                                .unwrap_or_else(|| "-".into());
                            if !ok {
                                api_errors::push(path, status);
                            }
                            rows.push(ProbeRow {
                                path: path.into(),
                                ok,
                                detail: format!("{status} encoding={encoding}"),
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

    let mut warnings = server.warnings.clone();
    if !server.version.is_empty() && server.version != frontend_version {
        warnings.push(ProdWarning {
            code: "VERSION_MISMATCH".into(),
            message: format!(
                "Frontend {frontend_version} ≠ backend {} — stale WASM or mixed deploy.",
                server.version
            ),
        });
    }

    let snapshot = json!({
        "role": role,
        "route": route_label,
        "theme": theme_label,
        "frontendVersion": frontend_version,
        "apiBase": api_base,
        "feedProvider": feed,
        "server": {
            "dbReachable": server.db_reachable,
            "dbTests": {
                "ping": { "ok": server.db_tests.ping.ok, "detail": server.db_tests.ping.detail },
                "listCollections": { "ok": server.db_tests.list_collections.ok, "detail": server.db_tests.list_collections.detail },
                "usersReadable": { "ok": server.db_tests.users_readable.ok, "detail": server.db_tests.users_readable.detail },
            },
            "rustLog": server.rust_log,
            "uptimeSecs": server.uptime_secs,
            "version": server.version,
            "buildId": server.build_id,
            "compressEnabled": server.compress_enabled,
        },
        "warnings": warnings,
        "recentErrors": &*errors,
        // Never include jwt / Authorization / secrets.
    });
    let snapshot_text = serde_json::to_string_pretty(&snapshot).unwrap_or_else(|_| "{}".into());

    let on_update_main = {
        let start_deploy = start_deploy.clone();
        Callback::from(move |_: MouseEvent| start_deploy.emit("main"))
    };
    let on_update_testing = {
        let start_deploy = start_deploy.clone();
        Callback::from(move |_: MouseEvent| start_deploy.emit("testing"))
    };
    let on_refresh_deploy = {
        let refresh_deploy = refresh_deploy.clone();
        Callback::from(move |_: MouseEvent| refresh_deploy.emit(()))
    };
    let job_running = *deploy_busy;
    let can_deploy = deploy.available && !job_running;

    html! {
        <div class="space-y-4">
            <div>
                <h2 class="title-sm">{ "Dev mode" }</h2>
                <p class="muted">
                    { "Admin-only live troubleshooting. Preference stays in this browser (localStorage). Not the install DEV_MODE flag." }
                </p>
            </div>

            <section class="card space-y-3">
                <div class="row-actions" style="align-items:center;justify-content:space-between;">
                    <p class="label-xs">{ "Deploy" }</p>
                    <button type="button" class="btn" onclick={on_refresh_deploy}>{ "Refresh" }</button>
                </div>
                <dl class="dev-kv">
                    <dt>{ "Build id" }</dt>
                    <dd>{ if deploy.build_id.is_empty() { "—" } else { deploy.build_id.as_str() } }</dd>
                    <dt>{ "Channel hint" }</dt>
                    <dd>{ if deploy.channel_hint.is_empty() { "—" } else { deploy.channel_hint.as_str() } }</dd>
                    <dt>{ "Control" }</dt>
                    <dd>{
                        if deploy.available {
                            if deploy.control_dir { "host path unit" } else { "local CLI" }
                        } else {
                            "unavailable"
                        }
                    }</dd>
                    <dt>{ "Autoupdate" }</dt>
                    <dd>{
                        match deploy.autoupdate_enabled {
                            Some(true) => format!(
                                "enabled ({})",
                                deploy.autoupdate_channel.clone().unwrap_or_else(|| "—".into())
                            ),
                            Some(false) => "disabled".into(),
                            None => "unknown (SSH: jheffmedia --autoupdate status)".into(),
                        }
                    }</dd>
                </dl>
                <p class="muted label-xs">{ &deploy.message }</p>
                <div class="row-actions">
                    <button
                        type="button"
                        class="btn"
                        disabled={!can_deploy}
                        onclick={on_update_main}
                    >{ "Update" }</button>
                    <button
                        type="button"
                        class="btn"
                        disabled={!can_deploy}
                        onclick={on_update_testing}
                    >{ "Update testing (-d)" }</button>
                </div>
                if let Some(msg) = &*deploy_note {
                    <p class="muted" role="status">{ msg }</p>
                }
                if let Some(job) = &*deploy_job {
                    <div class="space-y-2">
                        <p class="label-xs">{
                            {
                                format!(
                                    "Job {} — {} ({}){}",
                                    job.id,
                                    job.state,
                                    job.channel,
                                    job.exit_code
                                        .map(|c| format!(" exit={c}"))
                                        .unwrap_or_default()
                                )
                            }
                        }</p>
                        <textarea
                            class="field"
                            readonly=true
                            rows="8"
                            value={job.log_tail.clone()}
                            aria-label="Deploy job log"
                        />
                        <p class="muted label-xs">
                            { "Site may briefly restart mid-update; keep this tab open and wait for state ok/failed." }
                        </p>
                    </div>
                }
            </section>

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
                        <dt>{ "DB ping" }</dt><dd>{ format!("{} — {}", server.db_tests.ping.ok, server.db_tests.ping.detail) }</dd>
                        <dt>{ "DB collections" }</dt><dd>{ format!("{} — {}", server.db_tests.list_collections.ok, server.db_tests.list_collections.detail) }</dd>
                        <dt>{ "DB users" }</dt><dd>{ format!("{} — {}", server.db_tests.users_readable.ok, server.db_tests.users_readable.detail) }</dd>
                        <dt>{ "Compression" }</dt><dd>{ if server.compress_enabled { "enabled" } else { "disabled" } }</dd>
                        <dt>{ "RUST_LOG" }</dt><dd>{ if server.rust_log.is_empty() { "—" } else { server.rust_log.as_str() } }</dd>
                        <dt>{ "Uptime" }</dt><dd>{ format!("{}s", server.uptime_secs) }</dd>
                        <dt>{ "Backend version" }</dt><dd>{ if server.version.is_empty() { "—" } else { server.version.as_str() } }</dd>
                        <dt>{ "Build id" }</dt><dd>{ if server.build_id.is_empty() { "—" } else { server.build_id.as_str() } }</dd>
                    </dl>
                </section>

                if !warnings.is_empty() {
                    <section class="card space-y-2">
                        <p class="label-xs">{ "Production warnings" }</p>
                        <ul class="dev-list">
                            { for warnings.iter().map(|w| html! {
                                <li class="dev-bad">
                                    <code>{ &w.code }</code>
                                    { " — " }
                                    { &w.message }
                                </li>
                            }) }
                        </ul>
                    </section>
                }

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

async fn sleep_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        if let Some(win) = web_sys::window() {
            let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}
