use leptos::prelude::*;
use leptos_router::components::A;

#[server]
async fn get_dashboard_stats() -> Result<(i64, i64), ServerFnError> {
    let pool = crate::db::db().await?;
    let exercises = crate::db::count_exercises_db(&pool).await.unwrap_or(0);
    let workouts = crate::db::count_workouts_db(&pool).await.unwrap_or(0);
    Ok((exercises, workouts))
}

#[component]
pub fn HomePage() -> impl IntoView {
    let stats = Resource::new(|| (), |_| get_dashboard_stats());

    view! {
        <div class="home-page">
            <h1>"GritWit"</h1>
            <p class="tagline">"Body & Mind Tracker"</p>

            <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                {move || {
                    stats.get().map(|result| {
                        match result {
                            Ok((exercises, workouts)) => view! {
                                <div class="stats-grid">
                                    <div class="stat-card">
                                        <span class="stat-number">{exercises}</span>
                                        <span class="stat-label">"Exercises"</span>
                                    </div>
                                    <div class="stat-card">
                                        <span class="stat-number">{workouts}</span>
                                        <span class="stat-label">"Workouts"</span>
                                    </div>
                                </div>
                            }.into_any(),
                            Err(_) => view! { <p>"Error loading stats"</p> }.into_any(),
                        }
                    })
                }}
            </Suspense>

            <div class="quick-actions">
                <A href="/log" attr:class="btn btn-primary">"Log Workout"</A>
                <A href="/exercises" attr:class="btn btn-secondary">"Exercises"</A>
                <A href="/history" attr:class="btn btn-secondary">"History"</A>
            </div>
        </div>
    }
}
