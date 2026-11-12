use crate::{
    components::ui::{Heading, HeadingLevel, Spinner},
    model::ServerResponse,
    routes::AppRoutes,
    utils::{event_local_day, event_local_time, timezone_caption},
};
use gloo_net::http::Request;
use js_sys::Date;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ScheduleEvent {
    #[serde(default)]
    id: Option<String>,
    title: String,
    starts_at: String,
    #[serde(default)]
    location: Option<String>,
    #[serde(default)]
    kind: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ScheduleConfig {
    #[serde(default)]
    embed_url: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    events: Vec<ScheduleEvent>,
}

fn empty_schedule() -> ScheduleConfig {
    ScheduleConfig {
        embed_url: None,
        source: Some("none".into()),
        events: Vec::new(),
    }
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

fn format_day(day: &str) -> String {
    let Some((year, month)) = month_of(day) else {
        return day.to_string();
    };
    let date = day.rsplit('-').next().unwrap_or(day);
    let date = date.trim_start_matches('0');
    format!(
        "{} {date}, {year}",
        MONTHS.get(month as usize).copied().unwrap_or("")
    )
}

fn month_of(day: &str) -> Option<(i32, i32)> {
    let mut parts = day.split('-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: i32 = parts.next()?.parse().ok()?;
    Some((year, month - 1))
}

#[derive(Properties, PartialEq)]
struct CalendarProps {
    events: Vec<ScheduleEvent>,
    year: i32,
    month: i32,
    selected: Option<String>,
    on_shift: Callback<i32>,
    on_select: Callback<String>,
}

#[function_component(MonthCalendar)]
fn month_calendar(props: &CalendarProps) -> Html {
    let (today_y, today_m, today_d) = today();
    let blanks = first_weekday(props.year, props.month);
    let count = days_in_month(props.year, props.month);
    let label = format!(
        "{} {}",
        MONTHS.get(props.month as usize).copied().unwrap_or(""),
        props.year
    );

    let days = (1..=count).map(|day| {
        let key = day_key(props.year, props.month, day);
        let count = props
            .events
            .iter()
            .filter(|event| event_local_day(&event.starts_at).as_deref() == Some(key.as_str()))
            .count();
        let selected = props.selected.as_deref() == Some(key.as_str());
        let is_today = props.year == today_y && props.month == today_m && day == today_d;
        let mut class = String::from("calendar-day");
        if count > 0 {
            class.push_str(" has-event");
        }
        if is_today {
            class.push_str(" is-today");
        }
        if selected {
            class.push_str(" is-selected");
        }
        let on_select = props.on_select.clone();
        let key_for_click = key.clone();
        html! {
            <button
                type="button"
                class={class}
                aria-pressed={selected.to_string()}
                aria-label={format!("{} {day}", MONTHS.get(props.month as usize).copied().unwrap_or(""))}
                onclick={Callback::from(move |_| on_select.emit(key_for_click.clone()))}
            >
                <span>{ day.to_string() }</span>
                if count > 0 {
                    <span class="calendar-dot" />
                }
            </button>
        }
    });

    let on_prev = {
        let on_shift = props.on_shift.clone();
        Callback::from(move |_| on_shift.emit(-1))
    };
    let on_next = {
        let on_shift = props.on_shift.clone();
        Callback::from(move |_| on_shift.emit(1))
    };

    html! {
        <section class="calendar" aria-label="Availability calendar">
            <div class="calendar-bar">
                <button type="button" class="btn btn-quiet" onclick={on_prev}>{ "Prev" }</button>
                <p class="calendar-label">{ label }</p>
                <button type="button" class="btn btn-quiet" onclick={on_next}>{ "Next" }</button>
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
}

fn day_events(events: &[ScheduleEvent], day: &str) -> Vec<ScheduleEvent> {
    events
        .iter()
        .filter(|event| event_local_day(&event.starts_at).as_deref() == Some(day))
        .cloned()
        .collect()
}

#[function_component(Schedule)]
pub fn schedule() -> Html {
    // Show a blank calendar immediately; fill events when /api/schedule returns.
    let config = use_state(empty_schedule);
    let loading = use_state(|| true);
    let notice = use_state(|| None::<String>);
    let (start_y, start_m, _) = today();
    let cursor = use_state(|| (start_y, start_m));
    let selected = use_state(|| None::<String>);

    {
        let config = config.clone();
        let loading = loading.clone();
        let notice = notice.clone();
        let cursor = cursor.clone();
        let selected = selected.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match Request::get("/api/schedule").send().await {
                    Ok(resp) if resp.ok() => {
                        match resp.json::<ScheduleConfig>().await {
                            Ok(next) => {
                                if let Some(day) = next
                                    .events
                                    .iter()
                                    .find_map(|event| event_local_day(&event.starts_at))
                                {
                                    if let Some((year, month)) = month_of(&day) {
                                        cursor.set((year, month));
                                    }
                                    selected.set(Some(day.to_string()));
                                }
                                config.set(next);
                            }
                            Err(_) => {
                                notice.set(Some("Could not parse schedule config.".into()));
                            }
                        }
                    }
                    Ok(resp) => {
                        if let Ok(body) = resp.json::<ServerResponse<String>>().await {
                            notice.set(Some(body.message));
                        } else {
                            notice.set(Some("Schedule request failed.".into()));
                        }
                    }
                    Err(err) => notice.set(Some(err.to_string())),
                }
                loading.set(false);
            });
            || ()
        });
    }

    let (year, month) = *cursor;
    let on_shift = {
        let cursor = cursor.clone();
        let selected = selected.clone();
        Callback::from(move |delta: i32| {
            let (year, month) = *cursor;
            cursor.set(shift_month(year, month, delta));
            selected.set(None);
        })
    };
    let on_select = {
        let selected = selected.clone();
        Callback::from(move |day: String| selected.set(Some(day)))
    };
    let chosen = selected
        .as_ref()
        .map(|day| day_events(&config.events, day))
        .unwrap_or_default();

    html! {
        <div class="space-y-4">
            <Heading
                level={HeadingLevel::H1}
                subtitle="Upcoming availability. Days with a date are marked."
            >
                { "Schedule" }
            </Heading>
            if *loading {
                // Same overlay language as AuthModal; blocks interaction until /api/schedule returns.
                <div
                    class="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm"
                    role="dialog"
                    aria-modal="true"
                    aria-busy="true"
                    aria-label="Loading schedule"
                >
                    <div class="card">
                        <Spinner label="Loading schedule…" />
                    </div>
                </div>
            }
            if let Some(msg) = &*notice {
                <p class="muted">{ format!("Schedule note: {}", msg) }</p>
            }
            <MonthCalendar
                events={(*config).events.clone()}
                year={year}
                month={month}
                selected={(*selected).clone()}
                on_shift={on_shift}
                on_select={on_select}
            />
            <p class="muted">{ timezone_caption() }</p>
            if let Some(day) = &*selected {
                <section class="space-y-2">
                    <h2 class="title-sm">{ format_day(day) }</h2>
                    if chosen.is_empty() {
                        <p class="muted">{ "Nothing scheduled." }</p>
                    } else {
                        <ul class="space-y-2">
                            { for chosen.iter().map(|event| {
                                let location = event.location.clone().filter(|value| !value.is_empty());
                                let time = event_local_time(&event.starts_at);
                                html! {
                                    <li class="card">
                                        <p>{ event.title.clone() }</p>
                                        <p class="muted">{ format!("{} · {}", event.kind, if time.is_empty() { "All day" } else { time.as_str() }) }</p>
                                        if let Some(location) = location {
                                            <p class="muted">{ location }</p>
                                        }
                                    </li>
                                }
                            }) }
                        </ul>
                    }
                </section>
            } else if !(*loading) && config.events.is_empty() {
                <div class="space-y-3">
                    <p class="muted">{ "No upcoming dates. Send a booking request and we'll add one." }</p>
                    <Link<AppRoutes> to={AppRoutes::Booking} classes="btn btn-primary">
                        { "Request a date" }
                    </Link<AppRoutes>>
                </div>
            }
            if let Some(url) = &config.embed_url {
                <div class="card card-flat overflow-hidden">
                    <iframe
                        class="min-h-[min(70vh,720px)] w-full border-0 bg-surface"
                        src={url.clone()}
                        title="External calendar"
                        loading="lazy"
                    />
                </div>
            }
        </div>
    }
}
