use crate::db::WorkoutLog;
use crate::pages::wod::week_calendar::WeeklyCalendar;
use leptos::prelude::*;

#[server]
async fn get_history_for_date(date: String) -> Result<Vec<WorkoutLog>, ServerFnError> {
    use chrono::NaiveDate;

    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;

    let parsed: NaiveDate = date
        .parse()
        .map_err(|e| ServerFnError::new(format!("Invalid date: {}", e)))?;

    crate::db::list_workouts_by_date_db(&pool, user_uuid, parsed)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
pub fn HistoryPage() -> impl IntoView {
    let selected_date = RwSignal::new(String::new());

    let history = Resource::new(
        move || selected_date.get(),
        |date| async move {
            if date.is_empty() {
                Ok(vec![])
            } else {
                get_history_for_date(date).await
            }
        },
    );

    view! {
        <div class="history-page">
            <WeeklyCalendar selected_date=selected_date/>
            <Suspense fallback=|| ()>
                {move || history.get().map(|result| match result {
                    Ok(workouts) if workouts.is_empty() => view! {
                        <div class="empty-state">
                            <p class="empty-title">"No workouts"</p>
                            <p class="empty-sub">"Nothing logged for this day."</p>
                        </div>
                    }.into_any(),
                    Ok(workouts) => view! {
                        <div class="results-feed">
                            {workouts.into_iter().map(|w| {
                                let rx_label = if w.is_rx { "Rx" } else { "Scaled" };
                                let rx_class = if w.is_rx {
                                    "result-rx"
                                } else {
                                    "result-rx result-rx--scaled"
                                };
                                view! {
                                    <div class="result-card">
                                        <div class="result-card-header">
                                            <span class="result-date">{w.workout_date}</span>
                                            <span class={rx_class}>{rx_label}</span>
                                        </div>
                                        {w.notes.map(|n| view! {
                                            <div class="result-notes">{n}</div>
                                        })}
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any(),
                    Err(e) => view! {
                        <p class="error">{format!("Error: {}", e)}</p>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
