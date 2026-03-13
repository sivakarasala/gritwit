use leptos::prelude::*;

use super::get_week_dates;

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

#[component]
pub fn WeeklyCalendar(selected_date: RwSignal<String>) -> impl IntoView {
    let anchor = RwSignal::new(String::new());

    let week = Resource::new(move || anchor.get(), get_week_dates);

    Effect::new(move |_| {
        if let Some(Ok((today, _))) = week.get() {
            if selected_date.get_untracked().is_empty() {
                selected_date.set(today);
            }
        }
    });

    view! {
        <div class="week-calendar">
            <Suspense fallback=|| ()>
                {move || week.get().map(|res| match res {
                    Err(_) => ().into_any(),
                    Ok((today, dates)) => {
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
                        }.into_any()
                    }
                })}
            </Suspense>
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
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return date.to_string();
    }
    let Ok(y) = parts[0].parse::<i64>() else {
        return date.to_string();
    };
    let Ok(m) = parts[1].parse::<i64>() else {
        return date.to_string();
    };
    let Ok(d) = parts[2].parse::<i64>() else {
        return date.to_string();
    };

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
