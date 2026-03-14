use crate::db::LeaderboardEntry;
use leptos::prelude::*;

fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

#[component]
pub fn LeaderboardPreview(entries: Vec<LeaderboardEntry>) -> impl IntoView {
    view! {
        <div class="leaderboard-preview">
            <div class="leaderboard-header">
                <h3>"This Week"</h3>
                <span class="leaderboard-wod">"Leaderboard"</span>
            </div>
            {if entries.is_empty() {
                view! {
                    <div class="leaderboard-empty">
                        <p>"No workouts logged this week yet."</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="leaderboard-list">
                        {entries.into_iter().enumerate().map(|(i, entry)| {
                            let ini = initials(&entry.display_name);
                            let rank = i + 1;
                            let count_label = if entry.workout_count == 1 {
                                "workout".to_string()
                            } else {
                                "workouts".to_string()
                            };
                            view! {
                                <div class="leaderboard-entry">
                                    <span class="lb-rank">{rank}</span>
                                    <span class="lb-avatar">{ini}</span>
                                    <span class="lb-name">{entry.display_name}</span>
                                    <span class="lb-score">{format!("{} {}", entry.workout_count, count_label)}</span>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}
