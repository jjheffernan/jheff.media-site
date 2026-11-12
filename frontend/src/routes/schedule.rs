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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScheduleView {
    Weekly,
    Monthly,
    Seasonal,
    Events,
}

impl ScheduleView {
    fn label(self) -> &'static str {
        match self {
            Self::Weekly => "Weekly",
            Self::Monthly => "Monthly",
            Self::Seasonal => "Seasonal",
            Self::Events => "Events",
        }
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
const WEEKDAYS_FULL: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
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

fn shift_day(year: i32, month: i32, day: u32, delta: i32) -> (i32, i32, u32) {
    // Constructor accepts signed day and rolls across months (set_date is u32-only).
    let date = Date::new_with_year_month_day(year as u32, month, day as i32 + delta);
    (
        date.get_full_year() as i32,
        date.get_month() as i32,
        date.get_date(),
    )
}

fn weekday(year: i32, month: i32, day: u32) -> u32 {
    Date::new_with_year_month_day(year as u32, month, day as i32).get_day()
}

fn week_sunday(year: i32, month: i32, day: u32) -> (i32, i32, u32) {
    let dow = weekday(year, month, day) as i32;
    shift_day(year, month, day, -dow)
}

fn day_key(year: i32, month: i32, day: u32) -> String {
    format!("{year:04}-{:02}-{:02}", month + 1, day)
}

fn parse_day_key(day: &str) -> Option<(i32, i32, u32)> {
    let mut parts = day.split('-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: i32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    Some((year, month - 1, day))
}

fn format_day(day: &str) -> String {
    let Some((year, month, d)) = parse_day_key(day) else {
        return day.to_string();
    };
    format!(
        "{} {d}, {year}",
        MONTHS.get(month as usize).copied().unwrap_or("")
    )
}

/// Meteorological season: Winter Dec–Feb, Spring Mar–May, Summer Jun–Aug, Fall Sep–Nov.
fn season_index(month: i32) -> i32 {
    match month {
        11 | 0 | 1 => 0,
        2..=4 => 1,
        5..=7 => 2,
        _ => 3,
    }
}

/// Inclusive local-day range and label for the season containing `(year, month)`.
fn season_window(year: i32, month: i32) -> (String, String, String) {
    let season = season_index(month);
    match season {
        0 => {
            let start_year = if month == 11 { year } else { year - 1 };
            let end_year = start_year + 1;
            let end_day = days_in_month(end_year, 1);
            (
                day_key(start_year, 11, 1),
                day_key(end_year, 1, end_day),
                format!("Winter {start_year}–{end_year}"),
            )
        }
        1 => (
            day_key(year, 2, 1),
            day_key(year, 4, days_in_month(year, 4)),
            format!("Spring {year}"),
        ),
        2 => (
            day_key(year, 5, 1),
            day_key(year, 7, days_in_month(year, 7)),
            format!("Summer {year}"),
        ),
        _ => (
            day_key(year, 8, 1),
            day_key(year, 10, days_in_month(year, 10)),
            format!("Fall {year}"),
        ),
    }
}

fn shift_season(year: i32, month: i32, delta: i32) -> (i32, i32) {
    // Land on the middle month of the target season so labeling stays stable.
    let current = season_index(month);
    let next = current + delta;
    let year_delta = next.div_euclid(4);
    let season = next.rem_euclid(4);
    let mid_month = match season {
        0 => 0, // January (winter)
        1 => 3, // April
        2 => 6, // July
        _ => 9, // October
    };
    (year + year_delta, mid_month)
}

fn day_events(events: &[ScheduleEvent], day: &str) -> Vec<ScheduleEvent> {
    events
        .iter()
        .filter(|event| event_local_day(&event.starts_at).as_deref() == Some(day))
        .cloned()
        .collect()
}

fn events_in_range<'a>(events: &'a [ScheduleEvent], start: &str, end: &str) -> Vec<&'a ScheduleEvent> {
    events
        .iter()
        .filter(|event| {
            event_local_day(&event.starts_at)
                .map(|day| day.as_str() >= start && day.as_str() <= end)
                .unwrap_or(false)
        })
        .collect()
}

fn view_tab_class(active: bool) -> &'static str {
    if active {
        "btn btn-primary"
    } else {
        "btn btn-quiet"
    }
}

fn event_row(event: &ScheduleEvent, selected: bool, on_select: Callback<String>) -> Html {
    let day = event_local_day(&event.starts_at).unwrap_or_default();
    let location = event.location.clone().filter(|value| !value.is_empty());
    let time = event_local_time(&event.starts_at);
    let when = if time.is_empty() {
        "All day".to_string()
    } else {
        time
    };
    let mut class = String::from("schedule-event-row");
    if selected {
        class.push_str(" is-selected");
    }
    let day_for_click = day.clone();
    html! {
        <button
            type="button"
            class={class}
            aria-pressed={selected.to_string()}
            onclick={Callback::from(move |_| on_select.emit(day_for_click.clone()))}
        >
            <span class="schedule-event-when">
                <span>{ format_day(&day) }</span>
                <span class="muted">{ when }</span>
            </span>
            <span class="schedule-event-body">
                <span>{ event.title.clone() }</span>
                <span class="muted">{ event.kind.clone() }</span>
                if let Some(location) = location {
                    <span class="muted">{ location }</span>
                }
            </span>
        </button>
    }
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
        <section class="calendar" aria-label="Monthly calendar">
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

#[derive(Properties, PartialEq)]
struct WeekCalendarProps {
    events: Vec<ScheduleEvent>,
    year: i32,
    month: i32,
    day: u32,
    selected: Option<String>,
    on_shift: Callback<i32>,
    on_select: Callback<String>,
}

#[function_component(WeekCalendar)]
fn week_calendar(props: &WeekCalendarProps) -> Html {
    let (today_y, today_m, today_d) = today();
    let (sy, sm, sd) = week_sunday(props.year, props.month, props.day);
    let end = shift_day(sy, sm, sd, 6);
    let label = if sy == end.0 {
        format!(
            "{} {} – {} {}, {sy}",
            MONTHS.get(sm as usize).copied().unwrap_or(""),
            sd,
            MONTHS.get(end.1 as usize).copied().unwrap_or(""),
            end.2
        )
    } else {
        format!(
            "{} {} {sy} – {} {} {}",
            MONTHS.get(sm as usize).copied().unwrap_or(""),
            sd,
            MONTHS.get(end.1 as usize).copied().unwrap_or(""),
            end.2,
            end.0
        )
    };

    let days = (0..7).map(|offset| {
        let (y, m, d) = shift_day(sy, sm, sd, offset);
        let key = day_key(y, m, d);
        let day_events: Vec<_> = props
            .events
            .iter()
            .filter(|event| event_local_day(&event.starts_at).as_deref() == Some(key.as_str()))
            .cloned()
            .collect();
        let selected = props.selected.as_deref() == Some(key.as_str());
        let is_today = y == today_y && m == today_m && d == today_d;
        let mut class = String::from("calendar-week-day");
        if !day_events.is_empty() {
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
        let dow = WEEKDAYS_FULL
            .get(offset as usize)
            .copied()
            .unwrap_or("");
        html! {
            <button
                type="button"
                class={class}
                aria-pressed={selected.to_string()}
                aria-label={format!("{dow} {d}")}
                onclick={Callback::from(move |_| on_select.emit(key_for_click.clone()))}
            >
                <span class="calendar-week-dow">{ WEEKDAYS[offset as usize] }</span>
                <span class="calendar-week-num">{ d.to_string() }</span>
                if day_events.is_empty() {
                    <span class="muted calendar-week-empty">{ "—" }</span>
                } else {
                    <span class="calendar-week-titles">
                        { for day_events.iter().take(2).map(|event| {
                            html! { <span class="calendar-week-title">{ event.title.clone() }</span> }
                        }) }
                        if day_events.len() > 2 {
                            <span class="muted">{ format!("+{}", day_events.len() - 2) }</span>
                        }
                    </span>
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
        <section class="calendar calendar-week-view" aria-label="Weekly calendar">
            <div class="calendar-bar">
                <button type="button" class="btn btn-quiet" onclick={on_prev}>{ "Prev" }</button>
                <p class="calendar-label">{ label }</p>
                <button type="button" class="btn btn-quiet" onclick={on_next}>{ "Next" }</button>
            </div>
            <div class="calendar-week-strip">
                { for days }
            </div>
        </section>
    }
}

#[derive(Properties, PartialEq)]
struct SeasonViewProps {
    events: Vec<ScheduleEvent>,
    year: i32,
    month: i32,
    selected: Option<String>,
    on_shift: Callback<i32>,
    on_select: Callback<String>,
}

#[function_component(SeasonView)]
fn season_view(props: &SeasonViewProps) -> Html {
    let (start, end, label) = season_window(props.year, props.month);
    let season_events = events_in_range(&props.events, &start, &end);

    let on_prev = {
        let on_shift = props.on_shift.clone();
        Callback::from(move |_| on_shift.emit(-1))
    };
    let on_next = {
        let on_shift = props.on_shift.clone();
        Callback::from(move |_| on_shift.emit(1))
    };

    html! {
        <section class="calendar calendar-season-view" aria-label="Seasonal schedule">
            <div class="calendar-bar">
                <button type="button" class="btn btn-quiet" onclick={on_prev}>{ "Prev" }</button>
                <p class="calendar-label">{ label }</p>
                <button type="button" class="btn btn-quiet" onclick={on_next}>{ "Next" }</button>
            </div>
            <p class="muted">{ format!("{} – {}", format_day(&start), format_day(&end)) }</p>
            if season_events.is_empty() {
                <p class="muted">{ "Nothing scheduled this season." }</p>
            } else {
                <div class="schedule-event-list" role="list">
                    { for season_events.iter().map(|event| {
                        let day = event_local_day(&event.starts_at).unwrap_or_default();
                        let selected = props.selected.as_deref() == Some(day.as_str());
                        html! {
                            <div role="listitem">
                                { event_row(event, selected, props.on_select.clone()) }
                            </div>
                        }
                    }) }
                </div>
            }
        </section>
    }
}

#[derive(Properties, PartialEq)]
struct EventsViewProps {
    events: Vec<ScheduleEvent>,
    selected: Option<String>,
    on_select: Callback<String>,
}

#[function_component(EventsView)]
fn events_view(props: &EventsViewProps) -> Html {
    if props.events.is_empty() {
        return html! {
            <section class="calendar calendar-events-view" aria-label="Scheduled events">
                <p class="muted">{ "No upcoming dates." }</p>
            </section>
        };
    }

    html! {
        <section class="calendar calendar-events-view" aria-label="Scheduled events">
            <p class="calendar-label">{ "Upcoming dates" }</p>
            <div class="schedule-event-list" role="list">
                { for props.events.iter().map(|event| {
                    let day = event_local_day(&event.starts_at).unwrap_or_default();
                    let selected = props.selected.as_deref() == Some(day.as_str());
                    html! {
                        <div role="listitem">
                            { event_row(event, selected, props.on_select.clone()) }
                        </div>
                    }
                }) }
            </div>
        </section>
    }
}

fn selected_detail(day: &str, events: &[ScheduleEvent]) -> Html {
    let chosen = day_events(events, day);
    html! {
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
    }
}

#[function_component(Schedule)]
pub fn schedule() -> Html {
    // Show a blank calendar immediately; fill events when /api/schedule returns.
    let config = use_state(empty_schedule);
    let loading = use_state(|| true);
    let notice = use_state(|| None::<String>);
    let (start_y, start_m, start_d) = today();
    let view = use_state(|| ScheduleView::Monthly);
    let focus = use_state(|| (start_y, start_m, start_d));
    let selected = use_state(|| None::<String>);

    {
        let config = config.clone();
        let loading = loading.clone();
        let notice = notice.clone();
        let focus = focus.clone();
        let selected = selected.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                // ponytail: fetch full schedule once; later defer to visible month/range if payload grows.
                match Request::get("/api/schedule").send().await {
                    Ok(resp) if resp.ok() => {
                        match resp.json::<ScheduleConfig>().await {
                            Ok(next) => {
                                if let Some(day) = next
                                    .events
                                    .iter()
                                    .find_map(|event| event_local_day(&event.starts_at))
                                {
                                    if let Some((year, month, d)) = parse_day_key(&day) {
                                        focus.set((year, month, d));
                                    }
                                    selected.set(Some(day));
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

    let (year, month, day) = *focus;
    let on_select = {
        let selected = selected.clone();
        let focus = focus.clone();
        Callback::from(move |key: String| {
            if let Some((y, m, d)) = parse_day_key(&key) {
                focus.set((y, m, d));
            }
            selected.set(Some(key));
        })
    };

    let on_shift_month = {
        let focus = focus.clone();
        let selected = selected.clone();
        Callback::from(move |delta: i32| {
            let (year, month, _) = *focus;
            let (y, m) = shift_month(year, month, delta);
            focus.set((y, m, 1));
            selected.set(None);
        })
    };

    let on_shift_week = {
        let focus = focus.clone();
        let selected = selected.clone();
        Callback::from(move |delta: i32| {
            let (year, month, day) = *focus;
            let (sy, sm, sd) = week_sunday(year, month, day);
            let next = shift_day(sy, sm, sd, delta * 7);
            focus.set(next);
            selected.set(Some(day_key(next.0, next.1, next.2)));
        })
    };

    let on_shift_season = {
        let focus = focus.clone();
        let selected = selected.clone();
        Callback::from(move |delta: i32| {
            let (year, month, _) = *focus;
            let (y, m) = shift_season(year, month, delta);
            focus.set((y, m, 1));
            selected.set(None);
        })
    };

    let set_view = {
        let view = view.clone();
        Callback::from(move |next: ScheduleView| view.set(next))
    };

    let show_day_detail = matches!(
        *view,
        ScheduleView::Weekly | ScheduleView::Monthly | ScheduleView::Seasonal
    );

    html! {
        <div class="space-y-4">
            <Heading
                level={HeadingLevel::H1}
                subtitle="Upcoming availability. Days with a date are marked."
            >
                { "Schedule" }
            </Heading>
            if *loading {
                // Inline only — calendar/nav stay clickable while /api/schedule loads.
                <div class="schedule-load-status" role="status" aria-live="polite" aria-busy="true">
                    <Spinner label="Loading schedule…" />
                </div>
            }
            if let Some(msg) = &*notice {
                <p class="muted">{ format!("Schedule note: {}", msg) }</p>
            }
            <div class="schedule-view-switch" role="tablist" aria-label="Schedule views">
                { for [
                    ScheduleView::Weekly,
                    ScheduleView::Monthly,
                    ScheduleView::Seasonal,
                    ScheduleView::Events,
                ]
                .into_iter()
                .map(|item| {
                    let active = *view == item;
                    let set_view = set_view.clone();
                    html! {
                        <button
                            type="button"
                            role="tab"
                            class={view_tab_class(active)}
                            aria-selected={active.to_string()}
                            onclick={Callback::from(move |_| set_view.emit(item))}
                        >
                            { item.label() }
                        </button>
                    }
                }) }
            </div>
            {
                match *view {
                    ScheduleView::Weekly => html! {
                        <WeekCalendar
                            events={(*config).events.clone()}
                            year={year}
                            month={month}
                            day={day}
                            selected={(*selected).clone()}
                            on_shift={on_shift_week}
                            on_select={on_select.clone()}
                        />
                    },
                    ScheduleView::Monthly => html! {
                        <MonthCalendar
                            events={(*config).events.clone()}
                            year={year}
                            month={month}
                            selected={(*selected).clone()}
                            on_shift={on_shift_month}
                            on_select={on_select.clone()}
                        />
                    },
                    ScheduleView::Seasonal => html! {
                        <SeasonView
                            events={(*config).events.clone()}
                            year={year}
                            month={month}
                            selected={(*selected).clone()}
                            on_shift={on_shift_season}
                            on_select={on_select.clone()}
                        />
                    },
                    ScheduleView::Events => html! {
                        <EventsView
                            events={(*config).events.clone()}
                            selected={(*selected).clone()}
                            on_select={on_select.clone()}
                        />
                    },
                }
            }
            <p class="muted">{ timezone_caption() }</p>
            if show_day_detail {
                if let Some(day) = &*selected {
                    { selected_detail(day, &config.events) }
                } else if !(*loading) && config.events.is_empty() {
                    <div class="space-y-3">
                        <p class="muted">{ "No upcoming dates. Send a booking request and we'll add one." }</p>
                        <Link<AppRoutes> to={AppRoutes::Booking} classes="btn btn-primary">
                            { "Request a date" }
                        </Link<AppRoutes>>
                    </div>
                }
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
