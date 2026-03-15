mod leaderboard;

use crate::db::LeaderboardEntry;
use leaderboard::LeaderboardPreview;
use leptos::prelude::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DashboardData {
    pub day_name: String,
    pub full_date: String,
    pub exercises: i64,
    pub workouts: i64,
    pub streak: i64,
    pub leaderboard: Vec<LeaderboardEntry>,
}

#[server]
async fn get_dashboard() -> Result<DashboardData, ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;

    let exercises = crate::db::count_exercises_db(&pool).await.unwrap_or(0);
    let workouts = crate::db::count_workouts_db(&pool, user_uuid)
        .await
        .unwrap_or(0);
    let streak = crate::db::streak_days_db(&pool, user_uuid)
        .await
        .unwrap_or(0);
    let is_admin = matches!(user.role, crate::auth::UserRole::Admin);
    let leaderboard =
        crate::db::leaderboard_db(&pool, 5, user.email.as_deref().unwrap_or(""), is_admin)
            .await
            .unwrap_or_default();

    let today = chrono::Local::now().date_naive();
    let day_name = today.format("%A").to_string();
    let full_date = today.format("%B %-d, %Y").to_string();

    Ok(DashboardData {
        day_name,
        full_date,
        exercises,
        workouts,
        streak,
        leaderboard,
    })
}

#[component]
pub fn HomePage() -> impl IntoView {
    let dashboard = Resource::new(|| (), |_| get_dashboard());

    view! {
        <div class="home-page">
            <Suspense fallback=|| view! { <div class="home-page"><p class="loading">"Loading..."</p></div> }>
                {move || {
                    dashboard.get().map(|result| {
                        match result {
                            Ok(data) => {
                                let lb = data.leaderboard.clone();
                                view! {
                                    // Date header
                                    <div class="wod-date-header">
                                        <span class="wod-day">{data.day_name}</span>
                                        <span class="wod-full-date">{data.full_date}</span>
                                    </div>

                                    // Quick actions
                                    <div class="quick-actions">
                                        <a href="/log" class="quick-action-card">
                                            <span class="quick-action-icon">"+"</span>
                                            <span class="quick-action-label">"Log Workout"</span>
                                        </a>
                                        <a href="/exercises" class="quick-action-card">
                                            <span class="quick-action-icon">"☰"</span>
                                            <span class="quick-action-label">"Exercises"</span>
                                        </a>
                                        <a href="/history" class="quick-action-card">
                                            <span class="quick-action-icon">"↩"</span>
                                            <span class="quick-action-label">"History"</span>
                                        </a>
                                    </div>

                                    // Stats bar
                                    <div class="stats-bar">
                                        <div class="stats-bar-item">
                                            <span class="stats-bar-num">{data.workouts}</span>
                                            <span class="stats-bar-label">"Workouts"</span>
                                        </div>
                                        <div class="stats-bar-divider"></div>
                                        <div class="stats-bar-item">
                                            <span class="stats-bar-num">{data.exercises}</span>
                                            <span class="stats-bar-label">"Exercises"</span>
                                        </div>
                                        <div class="stats-bar-divider"></div>
                                        <div class="stats-bar-item">
                                            <span class="stats-bar-num">{data.streak}</span>
                                            <span class="stats-bar-label">"Day Streak"</span>
                                        </div>
                                    </div>

                                    // Leaderboard
                                    <LeaderboardPreview entries=lb/>
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
