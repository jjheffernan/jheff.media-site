use crate::{
    components::ui::{Alert, AlertVariant, Field},
    model::{ContactSubmission, ServerResponse},
    utils::{timezone_caption, with_local_offset},
};
use gloo_net::http::Request;
use js_sys::Date;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or(false)]
    pub booking: bool,
}

#[derive(Clone, PartialEq)]
enum SubmitState {
    Idle,
    Sending,
    Success(String),
    Error(String),
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const WEEKDAYS: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

fn today() -> (i32, i32, u32) {
    let now = Date::new_0();
    (
        now.get_full_year() as i32,
        now.get_month() as i32,
        now.get_date(),
    )
}

fn days_in_month(year: i32, month: i32) -> u32 {
    Date::new_with_year_month_day(year as u32, month + 1, 0).get_date()
}

fn first_weekday(year: i32, month: i32) -> u32 {
    Date::new_with_year_month_day(year as u32, month, 1).get_day()
}

fn shift_month(year: i32, month: i32, delta: i32) -> (i32, i32) {
    let index = year * 12 + month + delta;
    (index.div_euclid(12), index.rem_euclid(12))
}

fn day_key(year: i32, month: i32, day: u32) -> String {
    format!("{year:04}-{:02}-{:02}", month + 1, day)
}

fn combine_preferred(date: &str, time: &str) -> String {
    if date.is_empty() {
        return String::new();
    }
    let naive = if time.is_empty() {
        format!("{date}T12:00")
    } else {
        format!("{date}T{time}")
    };
    // RFC3339 with browser offset so preferredAt is not ambiguous.
    with_local_offset(&naive)
}

fn split_preferred(value: &str) -> (String, String) {
    if value.is_empty() {
        return (String::new(), String::new());
    }
    // Stored as RFC3339 or local `YYYY-MM-DDTHH:MM…`
    let date = value.get(..10).filter(|d| d.as_bytes().get(4) == Some(&b'-')).unwrap_or("").to_string();
    let time = value
        .split(['T', ' '])
        .nth(1)
        .map(|t| t.chars().take(5).collect::<String>())
        .filter(|t| t.len() == 5)
        .unwrap_or_default();
    (date, time)
}

fn display_line(date: &str, time: &str) -> String {
    match (date.is_empty(), time.is_empty()) {
        (true, true) => String::new(),
        (false, true) => date.to_string(),
        (true, false) => time.to_string(),
        (false, false) => format!("{date} {time}"),
    }
}

fn badge_date_label(date: &str) -> String {
    if date.len() == 10 {
        if let (Ok(y), Ok(m), Ok(d)) = (
            date[0..4].parse::<i32>(),
            date[5..7].parse::<usize>(),
            date[8..10].parse::<u32>(),
        ) {
            if (1..=12).contains(&m) {
                return format!(
                    "{} {d}",
                    MONTHS.get(m - 1).map(|n| &n[..3.min(n.len())]).unwrap_or("—")
                );
            }
            let _ = y;
        }
    }
    "Date".into()
}

/// Parse a typed line into (date, time). Accepts `YYYY-MM-DD[ T]HH:MM`, date-only, time-only.
fn parse_line(raw: &str) -> Option<(String, String)> {
    let s = raw.trim();
    if s.is_empty() {
        return Some((String::new(), String::new()));
    }
    let parts: Vec<&str> = s.split(|c: char| c.is_whitespace() || c == 'T').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return Some((String::new(), String::new()));
    }
    let mut date = String::new();
    let mut time = String::new();
    for part in parts {
        if part.contains(':') && time.is_empty() {
            let hhmm: String = part.chars().take(5).collect();
            if hhmm.len() == 5 && hhmm.as_bytes().get(2) == Some(&b':') {
                time = hhmm;
                continue;
            }
        }
        if date.is_empty() {
            // YYYY-MM-DD
            if part.len() == 10 && part.as_bytes().get(4) == Some(&b'-') {
                date = part.to_string();
                continue;
            }
            // M/D/YYYY or YYYY/M/D
            let segs: Vec<&str> = part.split(['/', '-', '.']).filter(|p| !p.is_empty()).collect();
            if segs.len() == 3 {
                let (y, m, d) = if segs[0].len() == 4 {
                    (
                        segs[0].parse::<i32>().ok()?,
                        segs[1].parse::<u32>().ok()?,
                        segs[2].parse::<u32>().ok()?,
                    )
                } else if segs[2].len() == 4 {
                    (
                        segs[2].parse::<i32>().ok()?,
                        segs[0].parse::<u32>().ok()?,
                        segs[1].parse::<u32>().ok()?,
                    )
                } else {
                    return None;
                };
                if (1..=12).contains(&m) && d >= 1 && d <= days_in_month(y, (m - 1) as i32) {
                    date = format!("{y:04}-{m:02}-{d:02}");
                    continue;
                }
            }
            return None;
        }
    }
    Some((date, time))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Panel {
    Closed,
    Date,
    Time,
}

#[derive(Properties, PartialEq)]
struct PreferredTimeProps {
    value: String,
    disabled: bool,
    on_change: Callback<String>,
}

/// One-line preferred datetime: type freely, or open Date / Time badges to pick.
#[function_component(PreferredTimePicker)]
fn preferred_time_picker(props: &PreferredTimeProps) -> Html {
    let (init_day, init_time) = split_preferred(&props.value);
    let selected_day = use_state(|| init_day);
    let selected_time = use_state(|| init_time);
    let draft = use_state(|| display_line(&*selected_day, &*selected_time));
    let panel = use_state(|| Panel::Closed);
    let (start_y, start_m, _) = today();
    let cursor = use_state(|| {
        let d = (*selected_day).clone();
        if d.len() >= 10 {
            if let (Ok(y), Ok(m)) = (d[0..4].parse::<i32>(), d[5..7].parse::<i32>()) {
                return (y, m - 1);
            }
        }
        (start_y, start_m)
    });

    {
        let selected_day = selected_day.clone();
        let selected_time = selected_time.clone();
        let draft = draft.clone();
        let value = props.value.clone();
        use_effect_with(value, move |value| {
            let (day, time) = split_preferred(value);
            selected_day.set(day.clone());
            selected_time.set(time.clone());
            draft.set(display_line(&day, &time));
            || ()
        });
    }

    let emit = {
        let on_change = props.on_change.clone();
        Callback::from(move |(day, time): (String, String)| {
            on_change.emit(combine_preferred(&day, &time));
        })
    };

    let on_input = {
        let draft = draft.clone();
        let selected_day = selected_day.clone();
        let selected_time = selected_time.clone();
        let emit = emit.clone();
        Callback::from(move |e: InputEvent| {
            let raw = e
                .target_unchecked_into::<web_sys::HtmlInputElement>()
                .value();
            draft.set(raw.clone());
            if let Some((day, time)) = parse_line(&raw) {
                selected_day.set(day.clone());
                selected_time.set(time.clone());
                emit.emit((day, time));
            }
        })
    };

    let on_blur = {
        let draft = draft.clone();
        let selected_day = selected_day.clone();
        let selected_time = selected_time.clone();
        let emit = emit.clone();
        Callback::from(move |_| {
            let line = display_line(&selected_day, &selected_time);
            draft.set(line);
            emit.emit(((*selected_day).clone(), (*selected_time).clone()));
        })
    };

    let toggle_date = {
        let panel = panel.clone();
        Callback::from(move |_| {
            panel.set(if *panel == Panel::Date {
                Panel::Closed
            } else {
                Panel::Date
            });
        })
    };
    let toggle_time = {
        let panel = panel.clone();
        Callback::from(move |_| {
            panel.set(if *panel == Panel::Time {
                Panel::Closed
            } else {
                Panel::Time
            });
        })
    };

    let on_prev = {
        let cursor = cursor.clone();
        Callback::from(move |_| {
            let (y, m) = *cursor;
            cursor.set(shift_month(y, m, -1));
        })
    };
    let on_next = {
        let cursor = cursor.clone();
        Callback::from(move |_| {
            let (y, m) = *cursor;
            cursor.set(shift_month(y, m, 1));
        })
    };

    let on_time = {
        let selected_time = selected_time.clone();
        let selected_day = selected_day.clone();
        let draft = draft.clone();
        let emit = emit.clone();
        Callback::from(move |e: InputEvent| {
            let time = e
                .target_unchecked_into::<web_sys::HtmlInputElement>()
                .value();
            selected_time.set(time.clone());
            draft.set(display_line(&selected_day, &time));
            emit.emit(((*selected_day).clone(), time));
        })
    };

    let on_clear = {
        let selected_day = selected_day.clone();
        let selected_time = selected_time.clone();
        let draft = draft.clone();
        let panel = panel.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |_| {
            selected_day.set(String::new());
            selected_time.set(String::new());
            draft.set(String::new());
            panel.set(Panel::Closed);
            on_change.emit(String::new());
        })
    };

    let (year, month) = *cursor;
    let blanks = first_weekday(year, month);
    let count = days_in_month(year, month);
    let label = format!(
        "{} {}",
        MONTHS.get(month as usize).copied().unwrap_or(""),
        year
    );
    let (today_y, today_m, today_d) = today();
    let day_set = (*selected_day).clone();
    let time_set = (*selected_time).clone();
    let has_value = !day_set.is_empty() || !time_set.is_empty();

    let days = (1..=count).map(|day| {
        let key = day_key(year, month, day);
        let selected = day_set.as_str() == key;
        let is_today = year == today_y && month == today_m && day == today_d;
        let mut class = String::from("calendar-day");
        if is_today {
            class.push_str(" is-today");
        }
        if selected {
            class.push_str(" is-selected");
        }
        let selected_day = selected_day.clone();
        let selected_time = selected_time.clone();
        let draft = draft.clone();
        let emit = emit.clone();
        let panel = panel.clone();
        let key_click = key.clone();
        let disabled = props.disabled;
        html! {
            <button
                type="button"
                class={class}
                disabled={disabled}
                aria-pressed={selected.to_string()}
                aria-label={format!("{} {day}", MONTHS.get(month as usize).copied().unwrap_or(""))}
                onclick={Callback::from(move |_| {
                    let time = if (*selected_time).is_empty() {
                        "12:30".into()
                    } else {
                        (*selected_time).clone()
                    };
                    selected_day.set(key_click.clone());
                    selected_time.set(time.clone());
                    draft.set(display_line(&key_click, &time));
                    emit.emit((key_click.clone(), time));
                    panel.set(Panel::Closed);
                })}
            >
                <span>{ day.to_string() }</span>
            </button>
        }
    });

    let date_badge_class = if *panel == Panel::Date {
        "datetime-badge is-active"
    } else if !day_set.is_empty() {
        "datetime-badge is-set"
    } else {
        "datetime-badge"
    };
    let time_badge_class = if *panel == Panel::Time {
        "datetime-badge is-active"
    } else if !time_set.is_empty() {
        "datetime-badge is-set"
    } else {
        "datetime-badge"
    };

    html! {
        <div class="preferred-time">
            <span class="label">{ "Preferred time" }</span>
            <div class="datetime-line">
                <button
                    type="button"
                    class={date_badge_class}
                    disabled={props.disabled}
                    aria-expanded={(*panel == Panel::Date).to_string()}
                    aria-label="Pick date"
                    onclick={toggle_date}
                >
                    { badge_date_label(&day_set) }
                </button>
                <button
                    type="button"
                    class={time_badge_class}
                    disabled={props.disabled}
                    aria-expanded={(*panel == Panel::Time).to_string()}
                    aria-label="Pick time"
                    onclick={toggle_time}
                >
                    { if time_set.is_empty() { "Time".into() } else { time_set.clone() } }
                </button>
                <input
                    type="text"
                    class="field datetime-input"
                    placeholder="Type date and time…"
                    value={(*draft).clone()}
                    oninput={on_input}
                    onblur={on_blur}
                    disabled={props.disabled}
                    autocomplete="off"
                    aria-label="Preferred date and time"
                />
                if has_value {
                    <button
                        type="button"
                        class="btn btn-ghost btn-sm"
                        disabled={props.disabled}
                        aria-label="Clear preferred time"
                        onclick={on_clear}
                    >
                        { "✕" }
                    </button>
                }
            </div>
            if *panel == Panel::Date {
                <section class="calendar datetime-pop" aria-label="Pick a date">
                    <div class="calendar-bar">
                        <button type="button" class="btn btn-quiet" disabled={props.disabled} onclick={on_prev}>{ "Prev" }</button>
                        <p class="calendar-label">{ label }</p>
                        <button type="button" class="btn btn-quiet" disabled={props.disabled} onclick={on_next}>{ "Next" }</button>
                    </div>
                    <div class="calendar-week">
                        { for WEEKDAYS.iter().map(|name| html! { <span class="calendar-dow">{ *name }</span> }) }
                    </div>
                    <div class="calendar-grid">
                        { for (0..blanks).map(|_| html! { <span /> }) }
                        { for days }
                    </div>
                </section>
            }
            if *panel == Panel::Time {
                <div class="datetime-pop datetime-time-pop">
                    <label class="label-xs">
                        { "Time" }
                        <input
                            type="time"
                            class="field"
                            value={if time_set.is_empty() { "12:30".into() } else { time_set.clone() }}
                            oninput={on_time}
                            disabled={props.disabled}
                        />
                    </label>
                </div>
            }
            <p class="form-hint">{ format!("{} · type YYYY-MM-DD HH:MM or use the badges.", timezone_caption()) }</p>
        </div>
    }
}

#[function_component(ContactForm)]
pub fn contact_form(props: &Props) -> Html {
    let name = use_state(String::new);
    let email = use_state(String::new);
    let subject = use_state(String::new);
    let message = use_state(String::new);
    let preferred = use_state(String::new);
    let website = use_state(String::new);
    let submit_state = use_state(|| SubmitState::Idle);
    let booking = props.booking;

    let on_name = {
        let name = name.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            name.set(input.value());
        })
    };
    let on_email = {
        let email = email.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            email.set(input.value());
        })
    };
    let on_subject = {
        let subject = subject.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            subject.set(input.value());
        })
    };
    let on_message = {
        let message = message.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>();
            message.set(input.value());
        })
    };

    let on_preferred = {
        let preferred = preferred.clone();
        Callback::from(move |value: String| {
            preferred.set(value);
        })
    };

    let on_website = {
        let website = website.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            website.set(input.value());
        })
    };

    let on_submit = {
        let name = name.clone();
        let email = email.clone();
        let subject = subject.clone();
        let message = message.clone();
        let preferred = preferred.clone();
        let website = website.clone();
        let submit_state = submit_state.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if (*name).trim().is_empty()
                || (*email).trim().is_empty()
                || (*message).trim().is_empty()
            {
                submit_state.set(SubmitState::Error(
                    "Name, email, and message are required.".into(),
                ));
                return;
            }
            submit_state.set(SubmitState::Sending);
            let payload = ContactSubmission {
                name: (*name).clone(),
                email: (*email).clone(),
                subject: (*subject).clone(),
                message: (*message).clone(),
                website: (*website).clone(),
                kind: if booking {
                    "booking".into()
                } else {
                    "contact".into()
                },
                preferred_at: if booking {
                    (*preferred).clone()
                } else {
                    String::new()
                },
            };
            let submit_state = submit_state.clone();
            spawn_local(async move {
                let body = serde_json::to_string(&payload).unwrap_or_default();
                let result = match Request::post("/api/contact")
                    .header("Content-Type", "application/json")
                    .body(body)
                {
                    Ok(req) => req.send().await,
                    Err(err) => Err(err),
                };
                match result {
                    Ok(resp) if resp.ok() => {
                        if booking {
                            submit_state.set(SubmitState::Success(
                                "Request received. We'll follow up.".into(),
                            ));
                            return;
                        }
                        if let Ok(body) = resp.json::<ServerResponse<String>>().await {
                            submit_state.set(SubmitState::Success(body.data));
                        } else {
                            submit_state.set(SubmitState::Success(
                                "Message sent. We'll get back to you soon.".into(),
                            ));
                        }
                    }
                    Ok(resp) => {
                        if let Ok(body) = resp.json::<ServerResponse<String>>().await {
                            let text = if body.data.is_empty() {
                                body.message
                            } else {
                                body.data
                            };
                            submit_state.set(SubmitState::Error(text));
                        } else {
                            submit_state.set(SubmitState::Error("Could not send message.".into()));
                        }
                    }
                    Err(err) => submit_state.set(SubmitState::Error(err.to_string())),
                }
            });
        })
    };

    let disabled = *submit_state == SubmitState::Sending;

    let feedback = match &*submit_state {
        SubmitState::Success(msg) => html! {
            <Alert variant={AlertVariant::Success} title={"Sent"}>{ html! { { msg.clone() } } }</Alert>
        },
        SubmitState::Error(msg) => html! {
            <Alert variant={AlertVariant::Destructive} title={"Could not send"}>{ html! { { msg.clone() } } }</Alert>
        },
        _ => html! {},
    };

    let subject_label = if booking { "What do you want" } else { "Subject" };

    html! {
        <form class="relative space-y-4" onsubmit={Callback::from(|e: SubmitEvent| e.prevent_default())}>
            <div class="absolute -left-[9999px] h-px w-px overflow-hidden" aria-hidden="true">
                <label>
                    { "Website" }
                    <input
                        type="text"
                        tabindex="-1"
                        autocomplete="off"
                        value={(*website).clone()}
                        oninput={on_website}
                    />
                </label>
            </div>
            <div class="grid gap-4 sm:grid-cols-2">
                <Field label={"Name"}>
                    <input
                        type="text"
                        class="field"
                        value={(*name).clone()}
                        oninput={on_name}
                        disabled={disabled}
                    />
                </Field>
                <Field label={"Email"}>
                    <input
                        type="email"
                        class="field"
                        value={(*email).clone()}
                        oninput={on_email}
                        disabled={disabled}
                    />
                </Field>
            </div>
            if !booking {
                <Field label={subject_label}>
                    <input
                        type="text"
                        class="field"
                        value={(*subject).clone()}
                        oninput={on_subject.clone()}
                        disabled={disabled}
                        placeholder="General question"
                    />
                </Field>
            }
            <Field label={"Message"}>
                <textarea
                    class="field field-area"
                    value={(*message).clone()}
                    oninput={on_message}
                    disabled={disabled}
                />
            </Field>
            if booking {
                <Field label={subject_label}>
                    <input
                        type="text"
                        class="field"
                        value={(*subject).clone()}
                        oninput={on_subject}
                        disabled={disabled}
                    />
                </Field>
                <PreferredTimePicker
                    value={(*preferred).clone()}
                    disabled={disabled}
                    on_change={on_preferred}
                />
                <p class="form-hint">{ "Optional. Leave it blank if you do not have a time." }</p>
            }
            { feedback }
            <button type="button" class="btn btn-primary" disabled={disabled} onclick={on_submit}>
                if disabled {
                    { "Sending…" }
                } else if booking {
                    { "Send request" }
                } else {
                    { "Send message" }
                }
            </button>
        </form>
    }
}
