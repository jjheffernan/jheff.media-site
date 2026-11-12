use js_sys::{Array, Date, Intl, Reflect};
use wasm_bindgen::JsValue;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// True when the stamp already carries `Z` or a numeric offset.
pub fn has_explicit_offset(value: &str) -> bool {
    let value = value.trim();
    value.len() >= 11
        && (value.contains('Z') || value.contains('+') || value[10..].contains('-'))
}

/// Append the browser's offset for `value`'s civil time (handles DST on that day).
/// Input like `2026-11-02T15:45` → `2026-11-02T15:45:00-05:00`.
pub fn with_local_offset(value: &str) -> String {
    let value = value.trim();
    if value.len() < 16 || has_explicit_offset(value) {
        return value.to_string();
    }
    let date = Date::new(&JsValue::from_str(value));
    if date.get_time().is_nan() {
        return value.to_string();
    }
    let offset_min = date.get_timezone_offset() as i32;
    // getTimezoneOffset is minutes west of UTC (EST → 300).
    let sign = if offset_min > 0 { '-' } else { '+' };
    let abs = offset_min.unsigned_abs();
    let stamp = if value.len() == 16 {
        format!("{value}:00")
    } else {
        value.to_string()
    };
    format!("{stamp}{sign}{:02}:{:02}", abs / 60, abs % 60)
}

fn utc_offset_label() -> String {
    let offset_min = Date::new_0().get_timezone_offset() as i32;
    let east = -offset_min;
    let sign = if east >= 0 { '+' } else { '−' };
    let abs = east.unsigned_abs();
    if abs % 60 == 0 {
        format!("UTC{sign}{}", abs / 60)
    } else {
        format!("UTC{sign}{}:{:02}", abs / 60, abs % 60)
    }
}

fn iana_timezone() -> Option<String> {
    let fmt = Intl::DateTimeFormat::new(&Array::new(), &js_sys::Object::new());
    let opts = fmt.resolved_options();
    Reflect::get(&opts, &JsValue::from_str("timeZone"))
        .ok()
        .and_then(|v| v.as_string())
}

/// e.g. `Times in America/New_York (UTC−5)` or `Times in UTC−5`.
pub fn timezone_caption() -> String {
    let offset = utc_offset_label();
    match iana_timezone() {
        Some(name) if !name.is_empty() => format!("Times in {name} ({offset})"),
        _ => format!("Times in {offset}"),
    }
}

/// Parse schedule/booking stamps. Naive values match backend: treated as UTC.
fn parse_instant(starts_at: &str) -> Option<Date> {
    let value = starts_at.trim();
    if value.is_empty() {
        return None;
    }
    let normalized = if has_explicit_offset(value) {
        value.to_string()
    } else if value.len() == 16 {
        // `YYYY-MM-DDTHH:MM`
        format!("{value}:00Z")
    } else if value.ends_with('Z') {
        value.to_string()
    } else {
        format!("{value}Z")
    };
    let date = Date::new(&JsValue::from_str(&normalized));
    if date.get_time().is_nan() {
        None
    } else {
        Some(date)
    }
}

/// Viewer-local calendar day `YYYY-MM-DD`.
pub fn event_local_day(starts_at: &str) -> Option<String> {
    let date = parse_instant(starts_at)?;
    Some(format!(
        "{:04}-{:02}-{:02}",
        date.get_full_year() as i32,
        date.get_month() as i32 + 1,
        date.get_date()
    ))
}

/// Viewer-local `HH:MM`.
pub fn event_local_time(starts_at: &str) -> String {
    let Some(date) = parse_instant(starts_at) else {
        return String::new();
    };
    format!("{:02}:{:02}", date.get_hours(), date.get_minutes())
}

/// Compact local stamp for lists, e.g. `2026-11-02 15:45`.
pub fn format_event_local(starts_at: &str) -> String {
    match (event_local_day(starts_at), event_local_time(starts_at)) {
        (Some(day), time) if !time.is_empty() => format!("{day} {time}"),
        (Some(day), _) => day,
        _ => starts_at.trim().to_string(),
    }
}
