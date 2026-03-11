use crate::db::WorkoutLog;
use leptos::prelude::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WeekDay {
    pub label: String,
    pub date_num: u32,
    pub full_date: String,
    pub is_today: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct HistoryData {
    pub week_label: String,
    pub week_days: Vec<WeekDay>,
    pub workouts: Vec<WorkoutLog>,
    pub user_name: String,
    pub week_offset: i64,
}

#[server]
async fn get_history(week_offset: i64, selected_date: Option<String>) -> Result<HistoryData, ServerFnError> {
    use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};

    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;

    let today = Local::now().date_naive();
    let days_since_monday = today.weekday().num_days_from_monday() as i64;
    let week_monday = today - Duration::days(days_since_monday) + Duration::weeks(week_offset);

    let week_days: Vec<WeekDay> = (0..7)
        .map(|i| {
            let d = week_monday + Duration::days(i);
            let label = match d.weekday() {
                Weekday::Mon => "M",
                Weekday::Tue => "T",
                Weekday::Wed => "W",
                Weekday::Thu => "T",
                Weekday::Fri => "F",
                Weekday::Sat => "S",
                Weekday::Sun => "S",
            }
            .to_string();
            WeekDay {
                label,
                date_num: d.day(),
                full_date: d.format("%Y-%m-%d").to_string(),
                is_today: d == today,
            }
        })
        .collect();

    // Build week label like "Mar 2026" or "Feb - Mar 2026" if week spans months
    let week_sunday = week_monday + Duration::days(6);
    let week_label = if week_monday.format("%b %Y").to_string() == week_sunday.format("%b %Y").to_string() {
        week_monday.format("%B %Y").to_string()
    } else if week_monday.year() == week_sunday.year() {
        format!("{} - {}", week_monday.format("%b"), week_sunday.format("%b %Y"))
    } else {
        format!("{} - {}", week_monday.format("%b %Y"), week_sunday.format("%b %Y"))
    };

    // If a specific date is selected, filter to that date; otherwise show all for the week
    let workouts = if let Some(ref date_str) = selected_date {
        let date: NaiveDate = date_str
            .parse()
            .map_err(|e| ServerFnError::new(format!("Invalid date: {}", e)))?;
        crate::db::list_workouts_by_date_db(&pool, user_uuid, date).await
    } else {
        sqlx::query_as::<_, WorkoutLog>(
            r#"SELECT
                id::text, workout_date::text, workout_type,
                name, notes, duration_seconds, is_rx
            FROM workout_logs
            WHERE user_id = $1 AND workout_date BETWEEN $2 AND $3
            ORDER BY workout_date DESC, created_at DESC"#,
        )
        .bind(user_uuid)
        .bind(week_monday)
        .bind(week_sunday)
        .fetch_all(&pool)
        .await
    }
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(HistoryData {
        week_label,
        week_days,
        workouts,
        user_name: user.display_name,
        week_offset,
    })
}

fn badge_class(wtype: &str) -> &'static str {
    match wtype {
        "amrap" => "result-badge--amrap",
        "strength" => "result-badge--strength",
        "emom" => "result-badge--emom",
        "for_time" => "result-badge--fortime",
        "meditation" => "result-badge--meditation",
        "breathing" => "result-badge--breathing",
        "chanting" => "result-badge--chanting",
        _ => "",
    }
}

fn user_initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

#[component]
pub fn HistoryPage() -> impl IntoView {
    let week_offset = RwSignal::new(0i64);
    let selected_date = RwSignal::new(None::<String>);

    let history = Resource::new(
        move || (week_offset.get(), selected_date.get()),
        |(offset, date)| get_history(offset, date),
    );

    view! {
        <div class="history-page">
            <Suspense fallback=|| view! { <div class="week-nav"><p>"Loading..."</p></div> }>
                {move || {
                    history.get().map(|result| {
                        match result {
                            Ok(data) => {
                                let days = data.week_days.clone();
                                let workouts = data.workouts.clone();
                                let uname = data.user_name.clone();
                                let ini = user_initials(&uname);

                                view! {
                                    // Month/year label
                                    <div class="week-month-label">{data.week_label}</div>

                                    // Week navigator
                                    <div class="week-nav">
                                        <button
                                            class="week-arrow"
                                            on:click=move |_| {
                                                week_offset.update(|v| *v -= 1);
                                                selected_date.set(None);
                                            }
                                        >"<"</button>
                                        <div class="week-days">
                                            {days.into_iter().map(|d| {
                                                let date_val = d.full_date.clone();
                                                let is_selected = selected_date.get() == Some(d.full_date.clone());
                                                let active_class = if d.is_today && selected_date.get().is_none() {
                                                    " active"
                                                } else if is_selected {
                                                    " active"
                                                } else {
                                                    ""
                                                };
                                                view! {
                                                    <div
                                                        class={format!("week-day{}", active_class)}
                                                        on:click={
                                                            let dv = date_val.clone();
                                                            move |_| {
                                                                if selected_date.get() == Some(dv.clone()) {
                                                                    selected_date.set(None);
                                                                } else {
                                                                    selected_date.set(Some(dv.clone()));
                                                                }
                                                            }
                                                        }
                                                        style="cursor:pointer"
                                                    >
                                                        {d.label}
                                                        <span class="week-date">{d.date_num}</span>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                        <button
                                            class="week-arrow"
                                            on:click=move |_| {
                                                week_offset.update(|v| *v += 1);
                                                selected_date.set(None);
                                            }
                                        >">"</button>
                                    </div>

                                    // Results feed
                                    {if workouts.is_empty() {
                                        view! {
                                            <div class="empty-state">
                                                <p class="empty-title">"No results"</p>
                                                <p class="empty-sub">"No workouts logged for this period."</p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="results-feed">
                                                {workouts.into_iter().map(|w| {
                                                    let duration_display = w.duration_seconds
                                                        .map(|s| {
                                                            let mins = s / 60;
                                                            let secs = s % 60;
                                                            if mins > 0 { format!("{}:{:02}", mins, secs) } else { format!("0:{:02}", secs) }
                                                        })
                                                        .unwrap_or_default();
                                                    let wtype = w.workout_type.clone();
                                                    let bcls = badge_class(&wtype);
                                                    let ini = ini.clone();
                                                    let uname = uname.clone();
                                                    let rx_label = if w.is_rx { "Rx" } else { "Scaled" };
                                                    let rx_class = if w.is_rx { "result-rx" } else { "result-rx result-rx--scaled" };
                                                    view! {
                                                        <div class="result-card">
                                                            <div class="result-card-header">
                                                                <div class="result-wod-info">
                                                                    <span class={format!("result-badge {}", bcls)}>{wtype.to_uppercase()}</span>
                                                                    {w.name.map(|n| view! { <span class="result-wod-name">{n}</span> })}
                                                                </div>
                                                                <span class="result-date">{w.workout_date}</span>
                                                            </div>
                                                            <div class="result-score-row">
                                                                <div class="result-avatar">{ini.clone()}</div>
                                                                <div class="result-details">
                                                                    <span class="result-athlete">{uname.clone()}</span>
                                                                    {(!duration_display.is_empty()).then(|| view! {
                                                                        <div class="result-score-line">
                                                                            <span class="result-score">{duration_display.clone()}</span>
                                                                        </div>
                                                                    })}
                                                                </div>
                                                                <span class={rx_class}>{rx_label}</span>
                                                            </div>
                                                            {w.notes.map(|n| view! { <div class="result-notes">{n}</div> })}
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }.into_any()
                                    }}
                                }.into_any()
                            }
                            Err(e) => view! { <p class="error">{format!("Error: {}", e)}</p> }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
