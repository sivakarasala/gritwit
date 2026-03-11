use crate::db::WorkoutLog;
use leptos::prelude::*;

#[server]
async fn list_workouts() -> Result<Vec<WorkoutLog>, ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user.id.parse().map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::list_workout_logs_db(&pool, user_uuid, 50)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
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

#[component]
pub fn HistoryPage() -> impl IntoView {
    let workouts = Resource::new(|| (), |_| list_workouts());

    view! {
        <div class="history-page">
            <h1>"Results"</h1>

            // Week navigator
            <div class="week-nav">
                <button class="week-arrow">"<"</button>
                <div class="week-days">
                    <div class="week-day">"M"<span class="week-date">"9"</span></div>
                    <div class="week-day">"T"<span class="week-date">"10"</span></div>
                    <div class="week-day active">"W"<span class="week-date">"11"</span></div>
                    <div class="week-day">"T"<span class="week-date">"12"</span></div>
                    <div class="week-day">"F"<span class="week-date">"13"</span></div>
                    <div class="week-day">"S"<span class="week-date">"14"</span></div>
                    <div class="week-day">"S"<span class="week-date">"15"</span></div>
                </div>
                <button class="week-arrow">">"</button>
            </div>

            <Suspense fallback=|| view! { <p class="loading">"Loading..."</p> }>
                {move || {
                    workouts.get().map(|result| {
                        match result {
                            Ok(list) if list.is_empty() => {
                                view! {
                                    <div class="results-feed">
                                        <div class="result-card">
                                            <div class="result-card-header">
                                                <div class="result-wod-info">
                                                    <span class="result-badge result-badge--amrap">"AMRAP"</span>
                                                    <span class="result-wod-name">"Cindy"</span>
                                                </div>
                                                <span class="result-date">"Today"</span>
                                            </div>
                                            <div class="result-score-row">
                                                <div class="result-avatar">"SK"</div>
                                                <div class="result-details">
                                                    <span class="result-athlete">"Siva Krishna"</span>
                                                    <div class="result-score-line">
                                                        <span class="result-score">"22 rounds + 5 reps"</span>
                                                        <span class="result-rx">"Rx"</span>
                                                    </div>
                                                </div>
                                                <span class="result-pr">"PR!"</span>
                                            </div>
                                            <div class="result-notes">"Felt strong today. Unbroken pull-ups through round 15."</div>
                                        </div>

                                        <div class="result-card">
                                            <div class="result-card-header">
                                                <div class="result-wod-info">
                                                    <span class="result-badge result-badge--strength">"Strength"</span>
                                                    <span class="result-wod-name">"Back Squat"</span>
                                                </div>
                                                <span class="result-date">"Today"</span>
                                            </div>
                                            <div class="result-score-row">
                                                <div class="result-avatar">"SK"</div>
                                                <div class="result-details">
                                                    <span class="result-athlete">"Siva Krishna"</span>
                                                    <div class="result-score-line">
                                                        <span class="result-score">"5-5-5-3-3-3 @ 100kg"</span>
                                                        <span class="result-rx">"Rx"</span>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        <div class="result-card">
                                            <div class="result-card-header">
                                                <div class="result-wod-info">
                                                    <span class="result-badge result-badge--meditation">"Meditation"</span>
                                                    <span class="result-wod-name">"Morning Stillness"</span>
                                                </div>
                                                <span class="result-date">"Yesterday"</span>
                                            </div>
                                            <div class="result-score-row">
                                                <div class="result-avatar">"SK"</div>
                                                <div class="result-details">
                                                    <span class="result-athlete">"Siva Krishna"</span>
                                                    <div class="result-score-line">
                                                        <span class="result-score">"20 min"</span>
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="result-notes">"Deep focus. Mantra: Om Namah Shivaya"</div>
                                        </div>

                                        <div class="result-card">
                                            <div class="result-card-header">
                                                <div class="result-wod-info">
                                                    <span class="result-badge result-badge--fortime">"For Time"</span>
                                                    <span class="result-wod-name">"Fran"</span>
                                                </div>
                                                <span class="result-date">"Mar 9"</span>
                                            </div>
                                            <div class="result-score-row">
                                                <div class="result-avatar">"SK"</div>
                                                <div class="result-details">
                                                    <span class="result-athlete">"Siva Krishna"</span>
                                                    <div class="result-score-line">
                                                        <span class="result-score">"3:42"</span>
                                                        <span class="result-rx">"Rx"</span>
                                                    </div>
                                                </div>
                                                <span class="result-pr">"PR!"</span>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                            Ok(list) => {
                                view! {
                                    <div class="results-feed">
                                        {list.into_iter().map(|w| {
                                            let duration_display = w.duration_seconds
                                                .map(|s| {
                                                    let mins = s / 60;
                                                    let secs = s % 60;
                                                    if mins > 0 { format!("{}:{:02}", mins, secs) } else { format!("0:{:02}", secs) }
                                                })
                                                .unwrap_or_default();
                                            let wtype = w.workout_type.clone();
                                            let bcls = badge_class(&wtype);
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
                                                        <div class="result-avatar">"SK"</div>
                                                        <div class="result-details">
                                                            <span class="result-athlete">"Siva Krishna"</span>
                                                            {(!duration_display.is_empty()).then(|| view! {
                                                                <div class="result-score-line">
                                                                    <span class="result-score">{duration_display}</span>
                                                                    <span class="result-rx">"Rx"</span>
                                                                </div>
                                                            })}
                                                        </div>
                                                    </div>
                                                    {w.notes.map(|n| view! { <div class="result-notes">{n}</div> })}
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
