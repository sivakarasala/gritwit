use leptos::prelude::*;

#[component]
pub fn LogWorkoutPage() -> impl IntoView {
    view! {
        <div class="log-workout-page">
            <div class="empty-state">
                <p class="empty-title">"Log Workout"</p>
                <p class="empty-sub">"WOD-driven logging coming soon."</p>
            </div>
        </div>
    }
}
