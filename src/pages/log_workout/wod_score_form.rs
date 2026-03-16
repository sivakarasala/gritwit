use crate::db::{MovementLog, MovementLogInput, SectionScoreInput, Wod, WodSection};
use leptos::prelude::*;

use super::section_score_card::{SectionScoreCard, SectionScoreState};

/// The actual scoring form for a WOD.
#[component]
pub fn WodScoreForm(
    wod: Wod,
    sections: Vec<WodSection>,
    focus_section: String,
    existing_scores: Vec<crate::db::SectionLog>,
    existing_movement_logs: Vec<MovementLog>,
    existing_notes: String,
    edit_log_id: RwSignal<String>,
) -> impl IntoView {
    let is_edit = Memo::new(move |_| !edit_log_id.get().is_empty());
    let submit_result = RwSignal::new(Option::<Result<String, String>>::None);
    let submitting = RwSignal::new(false);

    let workout_date = RwSignal::new(wod.programmed_date.clone());
    let overall_notes = RwSignal::new(existing_notes);

    // When a specific section is selected, only show that one
    let visible_sections: Vec<WodSection> = if !focus_section.is_empty() {
        sections
            .into_iter()
            .filter(|s| s.id == focus_section)
            .collect()
    } else {
        sections
    };

    // Create signals for each section, pre-populated from existing scores if editing
    let section_states: Vec<SectionScoreState> = visible_sections
        .iter()
        .map(|s| {
            let ex = existing_scores.iter().find(|sl| sl.section_id == s.id);
            // Filter movement logs belonging to this section's score log
            let section_mov_logs: Vec<MovementLog> = ex
                .map(|sl| {
                    existing_movement_logs
                        .iter()
                        .filter(|ml| ml.section_log_id == sl.id)
                        .cloned()
                        .collect()
                })
                .unwrap_or_default();
            SectionScoreState {
                section_id: s.id.clone(),
                section_type: s.section_type.clone(),
                title: s.title.clone().unwrap_or_else(|| s.section_type.clone()),
                time_cap: s.time_cap_minutes,
                rounds: s.rounds,
                is_rx: RwSignal::new(ex.map(|e| e.is_rx).unwrap_or(true)),
                skipped: RwSignal::new(ex.map(|e| e.skipped).unwrap_or(false)),
                minutes: RwSignal::new(
                    ex.and_then(|e| e.finish_time_seconds)
                        .map(|t| (t / 60).to_string())
                        .unwrap_or_default(),
                ),
                seconds: RwSignal::new(
                    ex.and_then(|e| e.finish_time_seconds)
                        .map(|t| (t % 60).to_string())
                        .unwrap_or_default(),
                ),
                rounds_completed: RwSignal::new(
                    ex.and_then(|e| e.rounds_completed)
                        .map(|r| r.to_string())
                        .unwrap_or_default(),
                ),
                extra_reps: RwSignal::new(
                    ex.and_then(|e| e.extra_reps)
                        .map(|r| r.to_string())
                        .unwrap_or_default(),
                ),
                weight_kg: RwSignal::new(
                    ex.and_then(|e| e.weight_kg)
                        .map(|w| w.to_string())
                        .unwrap_or_default(),
                ),
                notes: RwSignal::new(ex.and_then(|e| e.notes.clone()).unwrap_or_default()),
                movement_states: RwSignal::new(Vec::new()),
                existing_movement_logs: section_mov_logs,
            }
        })
        .collect();

    let section_states_submit = section_states.clone();
    let wod_id = wod.id.clone();
    let navigate = leptos_router::hooks::use_navigate();

    let on_submit = move |_| {
        submitting.set(true);
        submit_result.set(None);

        let scores: Vec<(SectionScoreInput, String)> = section_states_submit
            .iter()
            .map(|s| {
                let finish_time = if s.section_type == "fortime" {
                    let mins: i32 = s.minutes.get_untracked().parse().unwrap_or(0);
                    let secs: i32 = s.seconds.get_untracked().parse().unwrap_or(0);
                    let total = mins * 60 + secs;
                    if total > 0 {
                        Some(total)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let rounds_completed = if s.section_type == "amrap" || s.section_type == "emom" {
                    s.rounds_completed.get_untracked().parse().ok()
                } else {
                    None
                };
                let extra_reps = if s.section_type == "amrap" || s.section_type == "emom" {
                    s.extra_reps.get_untracked().parse().ok()
                } else {
                    None
                };
                let weight_kg = if s.section_type == "strength" {
                    s.weight_kg.get_untracked().parse().ok()
                } else {
                    None
                };
                let notes_val = s.notes.get_untracked();

                // Collect movement log inputs
                let movement_logs: Vec<MovementLogInput> = s
                    .movement_states
                    .get_untracked()
                    .iter()
                    .filter_map(|m| {
                        let (reps, sets, w) = if !m.set_rows.is_empty() {
                            // Aggregate from per-set rows
                            let total_reps: i32 = m
                                .set_rows
                                .iter()
                                .filter_map(|sr| sr.reps.get_untracked().parse::<i32>().ok())
                                .sum();
                            let num_sets = m.set_rows.len() as i32;
                            // Use max weight across sets
                            let max_weight: Option<f32> = m
                                .set_rows
                                .iter()
                                .filter_map(|sr| sr.weight_kg.get_untracked().parse::<f32>().ok())
                                .reduce(f32::max);
                            (
                                if total_reps > 0 {
                                    Some(total_reps)
                                } else {
                                    None
                                },
                                Some(num_sets),
                                max_weight,
                            )
                        } else {
                            (
                                m.reps.get_untracked().parse().ok(),
                                m.sets.get_untracked().parse().ok(),
                                m.weight_kg.get_untracked().parse().ok(),
                            )
                        };
                        let n = m.notes.get_untracked();
                        // Only include if at least one field is filled
                        if reps.is_some() || sets.is_some() || w.is_some() || !n.is_empty() {
                            Some(MovementLogInput {
                                movement_id: m.movement_id.clone(),
                                reps,
                                sets,
                                weight_kg: w,
                                notes: if n.is_empty() { None } else { Some(n) },
                            })
                        } else {
                            None
                        }
                    })
                    .collect();

                (
                    SectionScoreInput {
                        section_id: s.section_id.clone(),
                        finish_time_seconds: finish_time,
                        rounds_completed,
                        extra_reps,
                        weight_kg,
                        notes: if notes_val.is_empty() {
                            None
                        } else {
                            Some(notes_val)
                        },
                        is_rx: s.is_rx.get_untracked(),
                        skipped: s.skipped.get_untracked(),
                        movement_logs,
                    },
                    s.section_type.clone(),
                )
            })
            .collect();

        let scores_json = serde_json::to_string(&scores).unwrap_or_default();
        let wod_id = wod_id.clone();
        let date = workout_date.get_untracked();
        let notes = overall_notes.get_untracked();

        let log_id = edit_log_id.get_untracked();
        let nav = navigate.clone();
        leptos::task::spawn_local(async move {
            let nav_date = date.clone();
            let result = if log_id.is_empty() {
                super::server_fns::submit_wod_scores(wod_id, date, notes, scores_json)
                    .await
                    .map(|_| ())
            } else {
                super::server_fns::update_wod_scores(log_id, date, notes, scores_json).await
            };
            submitting.set(false);
            match result {
                Ok(_) => {
                    let msg = if is_edit.get_untracked() {
                        "Score updated!"
                    } else {
                        "Score logged!"
                    };
                    submit_result.set(Some(Ok(msg.to_string())));
                    set_timeout(
                        move || nav(&format!("/history?date={}", nav_date), Default::default()),
                        std::time::Duration::from_millis(800),
                    );
                }
                Err(e) => {
                    let msg = super::friendly_error(&e.to_string());
                    submit_result.set(Some(Err(msg)));
                }
            }
        });
    };

    view! {
        <div class="wod-score-form">
            <div class="score-header">
                <h2 class="score-wod-title">{wod.title.clone()}</h2>
                {wod.description.clone().map(|d| view! {
                    <p class="score-wod-desc">{d}</p>
                })}
            </div>

            <div class="score-sections">
                {section_states.into_iter().map(|state| {
                    let focused = state.section_id == focus_section;
                    view! { <SectionScoreCard state=state focused=focused/> }
                }).collect_view()}
            </div>

            <div class="score-footer">
                <div class="score-field">
                    <label class="score-label">"Notes (optional)"</label>
                    <textarea
                        class="score-textarea"
                        placeholder="How did it feel?"
                        prop:value=move || overall_notes.get()
                        on:input=move |ev| overall_notes.set(event_target_value(&ev))
                    ></textarea>
                </div>

                {move || submit_result.get().and_then(|r| r.err()).map(|e| view! {
                    <div class="score-inline-error">{e}</div>
                })}

                <button
                    class="score-submit"
                    class:btn--loading=move || submitting.get()
                    class:btn--success=move || matches!(submit_result.get(), Some(Ok(_)))
                    disabled=move || submitting.get() || matches!(submit_result.get(), Some(Ok(_)))
                    on:click=on_submit
                >
                    {move || if matches!(submit_result.get(), Some(Ok(_))) {
                        "\u{2713} Saved!".to_string()
                    } else if submitting.get() {
                        "Submitting...".to_string()
                    } else if is_edit.get() {
                        "Update Score".to_string()
                    } else {
                        "Log Score".to_string()
                    }}
                </button>
            </div>
        </div>
    }
}
