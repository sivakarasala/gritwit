use crate::db::{MovementLogWithName, SectionScoreWithMeta, WorkoutExercise};
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
    let movement_logs = entry.movement_logs.clone();

    let log_id_delete = log_id.clone();
    let navigate = leptos_router::hooks::use_navigate();

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
                            let navigate = navigate.clone();
                            move |_| {
                                // Scroll main to top instantly before navigating to avoid visible scroll
                                #[cfg(feature = "hydrate")]
                                {
                                    let _ = js_sys::eval(
                                        "var m=document.querySelector('main');if(m){m.scrollTo({top:0,behavior:'instant'})}",
                                    );
                                }
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

            // Section scores for WOD logs
            {if !entry.section_scores.is_empty() {
                let ml = movement_logs.clone();
                Some(view! {
                    <div class="result-sections">
                        {entry.section_scores.into_iter().map(|s| {
                            let label = s.section_title.clone()
                                .unwrap_or_else(|| format_section_type(&s.section_type));
                            let score = format_section_score(&s);
                            let rx_class = if s.is_rx { "result-rx" } else { "result-rx result-rx--scaled" };
                            let rx_label = if s.is_rx { "Rx" } else { "Scaled" };
                            let section_movements: Vec<_> = ml.iter()
                                .filter(|m| m.section_log_id == s.section_log_id)
                                .cloned()
                                .collect();
                            view! {
                                <div class="result-section-row">
                                    <span class="result-section-label">{label}</span>
                                    {(!s.skipped).then(|| view! {
                                        <span class={rx_class}>{rx_label}</span>
                                    })}
                                    <span class="result-section-score">
                                        {if s.skipped { "Skipped".to_string() } else { score }}
                                    </span>
                                </div>
                                {if !section_movements.is_empty() {
                                    Some(view! {
                                        <div class="result-movements">
                                            {section_movements.into_iter().map(|m| {
                                                let detail = format_movement(&m);
                                                view! {
                                                    <div class="result-movement-row">
                                                        <span class="result-movement-name">{m.exercise_name}</span>
                                                        <span class="result-movement-detail">{detail}</span>
                                                    </div>
                                                }
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

fn format_movement(m: &MovementLogWithName) -> String {
    let mut parts = Vec::new();
    if let Some(sets) = m.sets {
        if sets > 1 {
            parts.push(format!("{}×", sets));
        }
    }
    if let Some(reps) = m.reps {
        parts.push(format!("{} reps", reps));
    }
    if let Some(w) = m.weight_kg {
        parts.push(format!("{}kg", w));
    }
    if parts.is_empty() {
        "—".to_string()
    } else {
        parts.join(" ")
    }
}

fn format_section_type(t: &str) -> String {
    match t {
        "fortime" => "For Time".to_string(),
        "amrap" => "AMRAP".to_string(),
        "emom" => "EMOM".to_string(),
        "strength" => "Strength".to_string(),
        other => other.to_string(),
    }
}

fn format_section_score(s: &SectionScoreWithMeta) -> String {
    match s.section_type.as_str() {
        "fortime" => {
            if let Some(t) = s.finish_time_seconds {
                let mins = t / 60;
                let secs = t % 60;
                format!("{}:{:02}", mins, secs)
            } else {
                "—".to_string()
            }
        }
        "amrap" | "emom" => {
            let rounds = s.rounds_completed.unwrap_or(0);
            let reps = s.extra_reps.unwrap_or(0);
            if reps > 0 {
                format!("{} rounds + {} reps", rounds, reps)
            } else {
                format!("{} rounds", rounds)
            }
        }
        "strength" => {
            if let Some(w) = s.weight_kg {
                format!("{}kg", w)
            } else {
                "—".to_string()
            }
        }
        _ => "—".to_string(),
    }
}
