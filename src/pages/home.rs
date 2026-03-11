use leptos::prelude::*;
use leptos_router::components::A;

#[server]
async fn get_dashboard_stats() -> Result<(i64, i64), ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user.id.parse().map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let exercises = crate::db::count_exercises_db(&pool).await.unwrap_or(0);
    let workouts = crate::db::count_workouts_db(&pool, user_uuid).await.unwrap_or(0);
    Ok((exercises, workouts))
}

#[component]
pub fn HomePage() -> impl IntoView {
    let stats = Resource::new(|| (), |_| get_dashboard_stats());

    view! {
        <div class="home-page">
            // Date header
            <div class="wod-date-header">
                <span class="wod-day">"Tuesday"</span>
                <span class="wod-full-date">"March 11, 2026"</span>
            </div>

            // Today's WOD card
            <div class="wod-card">
                <div class="wod-card-header">
                    <span class="wod-badge wod-badge--amrap">"AMRAP"</span>
                    <span class="wod-time">"20 min"</span>
                </div>
                <h2 class="wod-title">"Cindy"</h2>
                <div class="wod-movements">
                    <div class="wod-movement">"5 Pull-ups"</div>
                    <div class="wod-movement">"10 Push-ups"</div>
                    <div class="wod-movement">"15 Air Squats"</div>
                </div>
                <div class="wod-footer">
                    <A href="/log" attr:class="wod-log-btn">"Log Result"</A>
                    <span class="wod-results-count">"3 results"</span>
                </div>
            </div>

            // Strength card
            <div class="wod-card wod-card--secondary">
                <div class="wod-card-header">
                    <span class="wod-badge wod-badge--strength">"Strength"</span>
                </div>
                <h2 class="wod-title">"Back Squat"</h2>
                <div class="wod-movements">
                    <div class="wod-movement">"5-5-5-3-3-3"</div>
                    <div class="wod-movement">"Build to a heavy triple"</div>
                </div>
                <div class="wod-footer">
                    <A href="/log" attr:class="wod-log-btn wod-log-btn--outline">"Log Result"</A>
                </div>
            </div>

            // Stats bar
            <Suspense fallback=|| view! { <div class="stats-bar">"Loading..."</div> }>
                {move || {
                    stats.get().map(|result| {
                        match result {
                            Ok((exercises, workouts)) => view! {
                                <div class="stats-bar">
                                    <div class="stats-bar-item">
                                        <span class="stats-bar-num">{workouts}</span>
                                        <span class="stats-bar-label">"Workouts"</span>
                                    </div>
                                    <div class="stats-bar-divider"></div>
                                    <div class="stats-bar-item">
                                        <span class="stats-bar-num">{exercises}</span>
                                        <span class="stats-bar-label">"Exercises"</span>
                                    </div>
                                    <div class="stats-bar-divider"></div>
                                    <div class="stats-bar-item">
                                        <span class="stats-bar-num">"7"</span>
                                        <span class="stats-bar-label">"Day Streak"</span>
                                    </div>
                                </div>
                            }.into_any(),
                            Err(_) => view! { <p>"Error loading stats"</p> }.into_any(),
                        }
                    })
                }}
            </Suspense>

            // Leaderboard preview
            <div class="leaderboard-preview">
                <div class="leaderboard-header">
                    <h3>"Leaderboard"</h3>
                    <span class="leaderboard-wod">"Cindy"</span>
                </div>
                <div class="leaderboard-list">
                    <div class="leaderboard-entry">
                        <span class="lb-rank">"1"</span>
                        <span class="lb-avatar">"SK"</span>
                        <span class="lb-name">"Siva Krishna"</span>
                        <span class="lb-score">"22 rds"</span>
                        <span class="lb-rx">"Rx"</span>
                    </div>
                    <div class="leaderboard-entry">
                        <span class="lb-rank">"2"</span>
                        <span class="lb-avatar">"AJ"</span>
                        <span class="lb-name">"Arjun M"</span>
                        <span class="lb-score">"19 rds"</span>
                        <span class="lb-rx">"Rx"</span>
                    </div>
                    <div class="leaderboard-entry">
                        <span class="lb-rank">"3"</span>
                        <span class="lb-avatar">"PR"</span>
                        <span class="lb-name">"Priya R"</span>
                        <span class="lb-score">"17 rds"</span>
                        <span class="lb-rx lb-rx--scaled">"Scaled"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
