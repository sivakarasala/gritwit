use crate::db::WorkoutExercise;
use leptos::prelude::*;

use super::HistoryEntry;

#[component]
pub fn HistoryCard(
    entry: HistoryEntry,
    show_delete: RwSignal<bool>,
    pending_delete_log_id: RwSignal<String>,
) -> impl IntoView {
    let is_wod = entry.log.wod_id.is_some();
    let log_id = entry.log.id.clone();
    let title = if let Some(ref t) = entry.wod_title {
        t.clone()
    } else {
        "Custom Workout".to_string()
    };
    let rx_label = if entry.log.is_rx { "Rx" } else { "Scaled" };
    let rx_class = if entry.log.is_rx {
        "result-rx"
    } else {
        "result-rx result-rx--scaled"
    };

    // Edit URL: for custom workouts go to /log?edit=<id>, for WOD workouts include log_id for pre-population
    let edit_url = if is_wod {
        format!(
            "/log?wod_id={}&edit_log={}",
            entry.log.wod_id.as_deref().unwrap_or_default(),
            log_id
        )
    } else {
        format!("/log?edit={}", log_id)
    };

    // Group exercises by name for display
    let exercise_groups = group_exercises(&entry.exercises);

    let log_id_delete = log_id.clone();

    view! {
        <div class="result-card">
            <div class="result-card-header">
                <div class="result-card-title-row">
                    <span class="result-title">{title}</span>
                    {is_wod.then(|| view! { <span class={rx_class}>{rx_label}</span> })}
                </div>
                <div class="result-card-actions">
                    <button
                        class="result-icon-btn result-icon-btn--edit"
                        title="Edit"
                        on:click={
                            let url = edit_url.clone();
                            move |_| {
                                let navigate = leptos_router::hooks::use_navigate();
                                navigate(&url, Default::default());
                            }
                        }
                    >"✎"</button>
                    <button
                        class="result-icon-btn result-icon-btn--delete"
                        title="Delete"
                        on:click=move |_| {
                            pending_delete_log_id.set(log_id_delete.clone());
                            show_delete.set(true);
                        }
                    >"×"</button>
                </div>
            </div>

            // Exercise details
            {if !exercise_groups.is_empty() {
                Some(view! {
                    <div class="result-exercises">
                        {exercise_groups
                            .into_iter()
                            .map(|(name, sets)| {
                                view! {
                                    <div class="result-exercise-group">
                                        <span class="result-exercise-name">{name}</span>
                                        <div class="result-sets">
                                            {sets
                                                .into_iter()
                                                .map(|set| {
                                                    let parts = format_set(&set);
                                                    view! {
                                                        <span class="result-set">{parts}</span>
                                                    }
                                                })
                                                .collect_view()}
                                        </div>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </div>
                })
            } else {
                None
            }}

            {entry.log.notes.map(|n| {
                view! { <div class="result-notes">{n}</div> }
            })}
        </div>
    }
}

/// Group exercises by name, preserving set order.
fn group_exercises(exercises: &[WorkoutExercise]) -> Vec<(String, Vec<WorkoutExercise>)> {
    let mut groups: Vec<(String, Vec<WorkoutExercise>)> = Vec::new();
    for ex in exercises {
        if let Some(group) = groups
            .iter_mut()
            .find(|(name, _)| *name == ex.exercise_name)
        {
            group.1.push(ex.clone());
        } else {
            groups.push((ex.exercise_name.clone(), vec![ex.clone()]));
        }
    }
    groups
}

/// Format a single set for display (e.g. "10 reps × 60kg").
fn format_set(set: &WorkoutExercise) -> String {
    let mut parts = Vec::new();
    if let Some(reps) = set.reps {
        parts.push(format!("{} reps", reps));
    }
    if let Some(weight) = set.weight_kg {
        parts.push(format!("{}kg", weight));
    }
    if let Some(dur) = set.duration_seconds {
        let mins = dur / 60;
        let secs = dur % 60;
        if mins > 0 {
            parts.push(format!("{}:{:02}", mins, secs));
        } else {
            parts.push(format!("{}s", secs));
        }
    }
    if parts.is_empty() {
        format!("Set {}", set.set_number)
    } else {
        parts.join(" × ")
    }
}
