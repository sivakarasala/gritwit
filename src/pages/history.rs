use crate::db::WorkoutLog;
use leptos::prelude::*;

#[server]
async fn list_workouts() -> Result<Vec<WorkoutLog>, ServerFnError> {
    let pool = crate::db::db().await?;
    crate::db::list_workout_logs_db(&pool, 50)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
pub fn HistoryPage() -> impl IntoView {
    let workouts = Resource::new(|| (), |_| list_workouts());

    view! {
        <div class="history-page">
            <h1>"Workout History"</h1>

            <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                {move || {
                    workouts.get().map(|result| {
                        match result {
                            Ok(list) if list.is_empty() => {
                                view! { <p class="empty">"No workouts logged yet."</p> }.into_any()
                            }
                            Ok(list) => {
                                view! {
                                    <div class="workout-list">
                                        {list.into_iter().map(|w| {
                                            let duration_display = w.duration_seconds
                                                .map(|s| {
                                                    let mins = s / 60;
                                                    let secs = s % 60;
                                                    if mins > 0 { format!("{}m {}s", mins, secs) } else { format!("{}s", secs) }
                                                })
                                                .unwrap_or_default();
                                            view! {
                                                <div class="workout-card">
                                                    <div class="workout-header">
                                                        <span class="workout-date">{w.workout_date}</span>
                                                        <span class="workout-type">{w.workout_type}</span>
                                                    </div>
                                                    {w.name.map(|n| view! { <div class="workout-name">{n}</div> })}
                                                    {(!duration_display.is_empty()).then(|| view! {
                                                        <div class="workout-duration">{duration_display}</div>
                                                    })}
                                                    {w.notes.map(|n| view! { <div class="workout-notes">{n}</div> })}
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
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
