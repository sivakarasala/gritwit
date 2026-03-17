use leptos::prelude::*;

/// Extract clientX from a touch event's touch list (e.g. "touches" or "changedTouches").
fn touch_client_x(_ev: &leptos::ev::TouchEvent, _list_name: &str) -> Option<f64> {
    #[cfg(feature = "hydrate")]
    {
        js_sys::Reflect::get(_ev, &_list_name.into())
            .ok()
            .and_then(|t| js_sys::Reflect::get(&t, &0.into()).ok())
            .and_then(|t| js_sys::Reflect::get(&t, &"clientX".into()).ok())
            .and_then(|v| v.as_f64())
    }
    #[cfg(not(feature = "hydrate"))]
    {
        None
    }
}

const DAY_LABELS: [&str; 7] = ["S", "M", "T", "W", "T", "F", "S"];

const MONTH_NAMES: [&str; 12] = [
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

/// Get today's date as YYYY-MM-DD string.
/// On the client (WASM) uses js_sys::Date; on the server uses chrono.
pub(crate) fn today_iso() -> String {
    #[cfg(feature = "hydrate")]
    {
        let d = js_sys::Date::new_0();
        format!(
            "{:04}-{:02}-{:02}",
            d.get_full_year(),
            d.get_month() + 1,
            d.get_date()
        )
    }
    #[cfg(feature = "ssr")]
    {
        chrono::Local::now().date_naive().to_string()
    }
    #[cfg(not(any(feature = "hydrate", feature = "ssr")))]
    {
        String::from("1970-01-01") // placeholder for cargo check without features
    }
}

/// Compute (today_iso, [sun..sat] iso strings) for the week containing `anchor`.
fn compute_week_dates(anchor: &str) -> (String, Vec<String>) {
    let today = today_iso();
    let (y, m, d) = if anchor.is_empty() {
        parse_ymd(&today)
    } else {
        parse_ymd(anchor)
    };
    let jdn = ymd_to_jdn(y, m, d);
    // day_of_week: 0=Mon,1=Tue,...,6=Sun  →  we want Sunday-start
    let dow = (jdn + 1) % 7; // 0=Sun,1=Mon,...,6=Sat
    let sunday_jdn = jdn - dow;
    let week: Vec<String> = (0..7)
        .map(|i| {
            let (ny, nm, nd) = jdn_to_ymd(sunday_jdn + i);
            format!("{:04}-{:02}-{:02}", ny, nm, nd)
        })
        .collect();
    (today, week)
}

fn parse_ymd(date: &str) -> (i64, i64, i64) {
    let parts: Vec<&str> = date.split('-').collect();
    let y = parts.first().and_then(|s| s.parse().ok()).unwrap_or(2026);
    let m = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let d = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    (y, m, d)
}

#[component]
pub fn WeeklyCalendar(selected_date: RwSignal<String>) -> impl IntoView {
    let anchor = RwSignal::new(String::new());

    // Compute week dates locally — no server call needed
    let week = Memo::new(move |_| compute_week_dates(&anchor.get()));

    // Track touch start X for swipe gesture
    let touch_start_x = RwSignal::new(0.0_f64);

    view! {
        <div class="week-calendar"
            on:touchstart=move |ev: leptos::ev::TouchEvent| {
                if let Some(x) = touch_client_x(&ev, "touches") {
                    touch_start_x.set(x);
                }
            }
            on:touchend=move |ev: leptos::ev::TouchEvent| {
                if let Some(end_x) = touch_client_x(&ev, "changedTouches") {
                    let dx = end_x - touch_start_x.get();
                    let threshold = 50.0;
                    if dx > threshold {
                        // Swiped right → previous week
                        let (_, dates) = week.get();
                        let first = dates.first().cloned().unwrap_or_default();
                        anchor.set(shift_date(&first, -7));
                    } else if dx < -threshold {
                        // Swiped left → next week
                        let (_, dates) = week.get();
                        let last = dates.last().cloned().unwrap_or_default();
                        anchor.set(shift_date(&last, 1));
                    }
                }
            }
        >
            {move || {
                let (today, dates) = week.get();
                let first = dates.first().cloned().unwrap_or_default();
                let last = dates.last().cloned().unwrap_or_default();
                let month_label = week_month_label(&first, &last);
                view! {
                    <div class="week-cal-month">{month_label}</div>
                    <div class="week-cal-row">
                        <button
                            class="week-cal-nav"
                            on:click=move |_| anchor.set(shift_date(&first, -7))
                        >"‹"</button>

                        <div class="week-cal-days">
                            {dates.into_iter().enumerate().map(|(i, date)| {
                                let day_num = date_day_num(&date);
                                let is_today = date == today;
                                let d = date.clone();
                                view! {
                                    <button
                                        class="week-cal-day"
                                        class:selected=move || selected_date.get() == d
                                        on:click={
                                            let d2 = date.clone();
                                            move |_| selected_date.set(d2.clone())
                                        }
                                    >
                                        <span class="week-cal-label">{DAY_LABELS[i]}</span>
                                        <span
                                            class="week-cal-num"
                                            class:today=is_today
                                        >{day_num}</span>
                                    </button>
                                }
                            }).collect_view()}
                        </div>

                        <button
                            class="week-cal-nav"
                            on:click=move |_| anchor.set(shift_date(&last, 1))
                        >"›"</button>
                    </div>
                }
            }}
        </div>
    }
}

fn date_day_num(date: &str) -> String {
    date.splitn(3, '-')
        .nth(2)
        .unwrap_or("?")
        .trim_start_matches('0')
        .to_string()
}

fn week_month_label(first: &str, last: &str) -> String {
    let (fy, fm) = parse_year_month(first);
    let (ly, lm) = parse_year_month(last);
    if fy == ly && fm == lm {
        format!("{} {}", MONTH_NAMES[(fm - 1) as usize], fy)
    } else if fy == ly {
        format!(
            "{} / {} {}",
            &MONTH_NAMES[(fm - 1) as usize][..3],
            &MONTH_NAMES[(lm - 1) as usize][..3],
            fy
        )
    } else {
        format!(
            "{} {} / {} {}",
            &MONTH_NAMES[(fm - 1) as usize][..3],
            fy,
            &MONTH_NAMES[(lm - 1) as usize][..3],
            ly
        )
    }
}

fn parse_year_month(date: &str) -> (i64, i64) {
    let parts: Vec<&str> = date.split('-').collect();
    let y = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let m = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    (y, m)
}

fn shift_date(date: &str, days: i64) -> String {
    let (y, m, d) = parse_ymd(date);
    let jdn = ymd_to_jdn(y, m, d) + days;
    let (ny, nm, nd) = jdn_to_ymd(jdn);
    format!("{:04}-{:02}-{:02}", ny, nm, nd)
}

fn ymd_to_jdn(y: i64, m: i64, d: i64) -> i64 {
    (1461 * (y + 4800 + (m - 14) / 12)) / 4 + (367 * (m - 2 - 12 * ((m - 14) / 12))) / 12
        - (3 * ((y + 4900 + (m - 14) / 12) / 100)) / 4
        + d
        - 32075
}

fn jdn_to_ymd(jdn: i64) -> (i64, i64, i64) {
    let l = jdn + 68569;
    let n = (4 * l) / 146097;
    let l = l - (146097 * n + 3) / 4;
    let i = (4000 * (l + 1)) / 1461001;
    let l = l - (1461 * i) / 4 + 31;
    let j = (80 * l) / 2447;
    let d = l - (2447 * j) / 80;
    let l = j / 11;
    let m = j + 2 - 12 * l;
    let y = 100 * (n - 49) + i + l;
    (y, m, d)
}
