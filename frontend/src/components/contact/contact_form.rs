use crate::{
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

#[derive(Properties, PartialEq)]
struct PreferredTimeProps {
    value: String,
    disabled: bool,
    on_change: Callback<String>,
}

#[function_component(PreferredTimePicker)]
fn preferred_time_picker(props: &PreferredTimeProps) -> Html {
    let (start_y, start_m, _) = today();
    let cursor = use_state(|| (start_y, start_m));
    let selected_day = use_state(|| {
        props
            .value
            .split('T')
            .next()
            .filter(|day| day.len() == 10)
            .unwrap_or("")
            .to_string()
    });
    let selected_time = use_state(|| {
        props
            .value
            .split('T')
            .nth(1)
            .map(|time| time.chars().take(5).collect::<String>())
            .unwrap_or_else(|| "12:30".into())
    });

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
        let on_change = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            let time = input.value();
            selected_time.set(time.clone());
            on_change.emit(combine_preferred(&selected_day, &time));
        })
    };

    let on_clear = {
        let selected_day = selected_day.clone();
        let selected_time = selected_time.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |_| {
            selected_day.set(String::new());
            selected_time.set("12:30".into());
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

    let days = (1..=count).map(|day| {
        let key = day_key(year, month, day);
        let selected = (*selected_day).as_str() == key;
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
        let on_change = props.on_change.clone();
        let key_for_click = key.clone();
        let disabled = props.disabled;
        html! {
            <button
                type="button"
                class={class}
                disabled={disabled}
                aria-pressed={selected.to_string()}
                aria-label={format!("{} {day}", MONTHS.get(month as usize).copied().unwrap_or(""))}
                onclick={Callback::from(move |_| {
                    selected_day.set(key_for_click.clone());
                    on_change.emit(combine_preferred(&key_for_click, &selected_time));
                })}
            >
                <span>{ day.to_string() }</span>
            </button>
        }
    });

    html! {
        <div class="preferred-time">
            <span class="label">{ "Preferred time" }</span>
            <section class="calendar" aria-label="Preferred date">
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
            <label class="label">
                { "Time" }
                <input
                    type="time"
                    class="field"
                    value={(*selected_time).clone()}
                    oninput={on_time}
                    disabled={props.disabled}
                />
            </label>
            <p class="muted">{ timezone_caption() }</p>
            if !(*selected_day).is_empty() {
                <button type="button" class="btn btn-quiet" disabled={props.disabled} onclick={on_clear}>
                    { "Clear" }
                </button>
            }
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
            <p class="muted">{ msg.clone() }</p>
        },
        SubmitState::Error(msg) => html! {
            <p class="muted">{ msg.clone() }</p>
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
                <label class="label">
                    { "Name" }
                    <input
                        type="text"
                        class="field"
                        value={(*name).clone()}
                        oninput={on_name}
                        disabled={disabled}
                    />
                </label>
                <label class="label">
                    { "Email" }
                    <input
                        type="email"
                        class="field"
                        value={(*email).clone()}
                        oninput={on_email}
                        disabled={disabled}
                    />
                </label>
            </div>
            if !booking {
                <label class="label">
                    { subject_label }
                    <input
                        type="text"
                        class="field"
                        value={(*subject).clone()}
                        oninput={on_subject.clone()}
                        disabled={disabled}
                        placeholder="General question"
                    />
                </label>
            }
            <label class="label">
                { "Message" }
                <textarea
                    class="field field-area"
                    value={(*message).clone()}
                    oninput={on_message}
                    disabled={disabled}
                />
            </label>
            if booking {
                <label class="label">
                    { subject_label }
                    <input
                        type="text"
                        class="field"
                        value={(*subject).clone()}
                        oninput={on_subject}
                        disabled={disabled}
                    />
                </label>
                <PreferredTimePicker
                    value={(*preferred).clone()}
                    disabled={disabled}
                    on_change={on_preferred}
                />
                <p class="muted">{ "Optional. Leave it blank if you do not have a time." }</p>
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
