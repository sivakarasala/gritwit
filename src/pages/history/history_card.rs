use crate::db::{MovementLogSet, MovementLogWithName, WorkoutExercise};
use leptos::prelude::*;

use super::editable_movement_row::EditableMovementRow;
use super::editable_section_row::EditableSectionRow;
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

    // Edit URL only for custom workouts (WODs use inline editing)
    let edit_url = if !is_wod {
        Some(format!("/log?edit={}", log_id))
    } else {
        None
    };

    // Group exercises by name for display
    let exercise_groups = group_exercises(&entry.exercises);
    let movement_logs = entry.movement_logs.clone();
    let movement_log_sets = entry.movement_log_sets.clone();

    let log_id_delete = log_id.clone();
    let navigate = leptos_router::hooks::use_navigate();

    view! {
        <div class="result-card">
            <div class="result-card-header">
                <div class="result-card-title-row">
                    <span class="result-title">{title}</span>
                </div>
                <div class="result-card-actions">
                    {edit_url.map(|url| {
                        let navigate = navigate.clone();
                        view! {
                            <button
                                class="result-icon-btn result-icon-btn--edit"
                                title="Edit"
                                on:click=move |_| {
                                    #[cfg(feature = "hydrate")]
                                    {
                                        let _ = js_sys::eval(
                                            "var m=document.querySelector('main');if(m){m.scrollTo({top:0,behavior:'instant'})}",
                                        );
                                    }
                                    navigate(&url, Default::default());
                                }
                            >"✎"</button>
                        }
                    })}
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

            // Section scores for WOD logs — each section is inline-editable
            {if !entry.section_scores.is_empty() {
                let ml = movement_logs.clone();
                let mls = movement_log_sets.clone();
                Some(view! {
                    <div class="result-sections">
                        {entry.section_scores.into_iter().map(|s| {
                            let section_movements: Vec<MovementLogWithName> = ml.iter()
                                .filter(|m| m.section_log_id == s.section_log_id)
                                .cloned()
                                .collect();
                            let all_sets = mls.clone();
                            view! {
                                <EditableSectionRow s=s/>
                                {if !section_movements.is_empty() {
                                    Some(view! {
                                        <div class="result-movements">
                                            {section_movements.into_iter().map(|m| {
                                                let sets: Vec<MovementLogSet> = all_sets.iter()
                                                    .filter(|s| s.movement_log_id == m.id)
                                                    .cloned()
                                                    .collect();
                                                view! { <EditableMovementRow m=m sets=sets/> }
                                            }).collect_view()}
                                        </div>
                                    })
                                } else {
                                    None
                                }}
                            }
                        }).collect_view()}
                    </div>
                })
            } else {
                None
            }}

            // Exercise details for custom workouts
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

/// Format a single custom workout set for display.
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
    if let Some(dist) = set.distance_meters {
        parts.push(format!("{}m", dist));
    }
    if let Some(cal) = set.calories {
        parts.push(format!("{} cal", cal));
    }
    if parts.is_empty() {
        format!("Set {}", set.set_number)
    } else {
        parts.join(" × ")
    }
}
