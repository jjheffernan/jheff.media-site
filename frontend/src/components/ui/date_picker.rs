//! Typeable date field + month calendar. Value is `YYYY-MM-DD` when valid.
use js_sys::Date;
use web_sys::HtmlInputElement;
use yew::prelude::*;

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

/// Accept `YYYY-MM-DD` or common loose forms; return canonical or None.
fn normalize_date(raw: &str) -> Option<String> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    // YYYY-MM-DD
    if s.len() == 10 && s.as_bytes().get(4) == Some(&b'-') && s.as_bytes().get(7) == Some(&b'-') {
        let y: i32 = s[0..4].parse().ok()?;
        let m: u32 = s[5..7].parse().ok()?;
        let d: u32 = s[8..10].parse().ok()?;
        if (1..=12).contains(&m) && d >= 1 && d <= days_in_month(y, (m - 1) as i32) {
            return Some(format!("{y:04}-{m:02}-{d:02}"));
        }
        return None;
    }
    // YYYY/MM/DD or MM/DD/YYYY
    let parts: Vec<&str> = s.split(['/', '-', '.']).filter(|p| !p.is_empty()).collect();
    if parts.len() != 3 {
        return None;
    }
    let (y, m, d) = if parts[0].len() == 4 {
        (
            parts[0].parse::<i32>().ok()?,
            parts[1].parse::<u32>().ok()?,
            parts[2].parse::<u32>().ok()?,
        )
    } else if parts[2].len() == 4 {
        (
            parts[2].parse::<i32>().ok()?,
            parts[0].parse::<u32>().ok()?,
            parts[1].parse::<u32>().ok()?,
        )
    } else {
        return None;
    };
    if (1..=12).contains(&m) && d >= 1 && d <= days_in_month(y, (m - 1) as i32) {
        Some(format!("{y:04}-{m:02}-{d:02}"))
    } else {
        None
    }
}

fn cursor_from_value(value: &str) -> (i32, i32) {
    if let Some(norm) = normalize_date(value) {
        if let (Ok(y), Ok(m)) = (norm[0..4].parse::<i32>(), norm[5..7].parse::<i32>()) {
            return (y, m - 1);
        }
    }
    let (y, m, _) = today();
    (y, m)
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub value: String,
    pub on_change: Callback<String>,
    #[prop_or_else(|| "Date".into())]
    pub placeholder: String,
    #[prop_or(false)]
    pub disabled: bool,
}

#[function_component(DatePicker)]
pub fn date_picker(props: &Props) -> Html {
    let open = use_state(|| false);
    let draft = use_state(|| props.value.clone());
    let cursor = use_state(|| cursor_from_value(&props.value));

    {
        let draft = draft.clone();
        let cursor = cursor.clone();
        let value = props.value.clone();
        use_effect_with(value, move |value| {
            draft.set(value.clone());
            cursor.set(cursor_from_value(value));
            || ()
        });
    }

    let on_input = {
        let draft = draft.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let raw = e.target_unchecked_into::<HtmlInputElement>().value();
            draft.set(raw.clone());
            if let Some(norm) = normalize_date(&raw) {
                on_change.emit(norm);
            } else if raw.trim().is_empty() {
                on_change.emit(String::new());
            } else {
                // Keep free text while typing; parent still gets last valid or raw on blur.
                on_change.emit(raw);
            }
        })
    };

    let on_blur = {
        let draft = draft.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |_| {
            let raw = (*draft).clone();
            if let Some(norm) = normalize_date(&raw) {
                draft.set(norm.clone());
                on_change.emit(norm);
            } else if raw.trim().is_empty() {
                on_change.emit(String::new());
            }
        })
    };

    let toggle = {
        let open = open.clone();
        Callback::from(move |_| open.set(!*open))
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

    let (year, month) = *cursor;
    let blanks = first_weekday(year, month);
    let count = days_in_month(year, month);
    let label = format!(
        "{} {}",
        MONTHS.get(month as usize).copied().unwrap_or(""),
        year
    );
    let (today_y, today_m, today_d) = today();
    let selected = normalize_date(&props.value).unwrap_or_default();

    let days = (1..=count).map(|day| {
        let key = day_key(year, month, day);
        let is_selected = selected.as_str() == key;
        let is_today = year == today_y && month == today_m && day == today_d;
        let mut class = String::from("calendar-day");
        if is_today {
            class.push_str(" is-today");
        }
        if is_selected {
            class.push_str(" is-selected");
        }
        let draft = draft.clone();
        let on_change = props.on_change.clone();
        let open = open.clone();
        let key_click = key.clone();
        let disabled = props.disabled;
        html! {
            <button
                type="button"
                class={class}
                disabled={disabled}
                aria-pressed={is_selected.to_string()}
                aria-label={format!("{} {day}", MONTHS.get(month as usize).copied().unwrap_or(""))}
                onclick={Callback::from(move |_| {
                    draft.set(key_click.clone());
                    on_change.emit(key_click.clone());
                    open.set(false);
                })}
            >
                <span>{ day.to_string() }</span>
            </button>
        }
    });

    html! {
        <div class="date-picker">
            <div class="date-picker-row">
                <input
                    type="text"
                    class="field"
                    placeholder={props.placeholder.clone()}
                    value={(*draft).clone()}
                    oninput={on_input}
                    onblur={on_blur}
                    disabled={props.disabled}
                    inputmode="numeric"
                    autocomplete="off"
                    aria-label="Date"
                />
                <button
                    type="button"
                    class="btn btn-quiet date-picker-toggle"
                    disabled={props.disabled}
                    aria-expanded={(*open).to_string()}
                    aria-label="Toggle calendar"
                    onclick={toggle}
                >
                    { "Cal" }
                </button>
            </div>
            if *open {
                <section class="calendar" aria-label="Pick a date">
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
                    <p class="muted label-xs">{ "Type YYYY-MM-DD or pick a day." }</p>
                </section>
            }
        </div>
    }
}
