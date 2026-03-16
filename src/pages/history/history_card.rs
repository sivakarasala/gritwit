use crate::db::{MovementLogSet, MovementLogWithName, SectionScoreWithMeta, WorkoutExercise};
use leptos::prelude::*;

use super::{update_movement_log, update_movement_log_set, update_section_score, HistoryEntry};

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
                            let section_movements: Vec<_> = ml.iter()
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

#[component]
fn EditableSectionRow(s: SectionScoreWithMeta) -> impl IntoView {
    let editing = RwSignal::new(false);
    let saving = RwSignal::new(false);
    let save_result = RwSignal::new(Option::<Result<(), String>>::None);

    let section_log_id = s.section_log_id.clone();
    let section_type = s.section_type.clone();
    let label = s
        .section_title
        .clone()
        .unwrap_or_else(|| format_section_type(&s.section_type));

    // Display signals
    let display_score = RwSignal::new(if s.skipped {
        "Skipped".to_string()
    } else {
        format_section_score(&s)
    });
    let display_rx = RwSignal::new(s.is_rx);

    // Edit state based on section type
    let minutes = RwSignal::new(
        s.finish_time_seconds
            .map(|t| (t / 60).to_string())
            .unwrap_or_default(),
    );
    let seconds = RwSignal::new(
        s.finish_time_seconds
            .map(|t| (t % 60).to_string())
            .unwrap_or_default(),
    );
    let rounds = RwSignal::new(
        s.rounds_completed
            .map(|r| r.to_string())
            .unwrap_or_default(),
    );
    let extra_reps = RwSignal::new(s.extra_reps.map(|r| r.to_string()).unwrap_or_default());
    let weight = RwSignal::new(s.weight_kg.map(|w| w.to_string()).unwrap_or_default());
    let is_rx = RwSignal::new(s.is_rx);

    // Original values for cancel
    let orig_minutes = s
        .finish_time_seconds
        .map(|t| (t / 60).to_string())
        .unwrap_or_default();
    let orig_seconds = s
        .finish_time_seconds
        .map(|t| (t % 60).to_string())
        .unwrap_or_default();
    let orig_rounds = s
        .rounds_completed
        .map(|r| r.to_string())
        .unwrap_or_default();
    let orig_extra_reps = s.extra_reps.map(|r| r.to_string()).unwrap_or_default();
    let orig_weight = s.weight_kg.map(|w| w.to_string()).unwrap_or_default();
    let orig_rx = s.is_rx;

    let st = section_type.clone();
    let on_save = {
        let sl_id = section_log_id.clone();
        let st = st.clone();
        move |_| {
            saving.set(true);
            save_result.set(None);

            let finish_time = match st.as_str() {
                "fortime" => {
                    let m: i32 = minutes.get_untracked().parse().unwrap_or(0);
                    let s: i32 = seconds.get_untracked().parse().unwrap_or(0);
                    let total = m * 60 + s;
                    if total > 0 {
                        Some(total)
                    } else {
                        None
                    }
                }
                _ => None,
            };
            let rounds_val: Option<i32> = match st.as_str() {
                "amrap" | "emom" => rounds.get_untracked().parse().ok(),
                _ => None,
            };
            let extra_reps_val: Option<i32> = match st.as_str() {
                "amrap" | "emom" => {
                    let v: Option<i32> = extra_reps.get_untracked().parse().ok();
                    v.filter(|&r| r > 0)
                }
                _ => None,
            };
            let weight_val: Option<f32> = match st.as_str() {
                "strength" => weight.get_untracked().parse().ok(),
                _ => None,
            };
            let rx = is_rx.get_untracked();

            let sl_id = sl_id.clone();
            let st2 = st.clone();
            leptos::task::spawn_local(async move {
                let result = update_section_score(
                    sl_id,
                    finish_time,
                    rounds_val,
                    extra_reps_val,
                    weight_val,
                    rx,
                )
                .await;
                saving.set(false);
                match result {
                    Ok(_) => {
                        let new_score = format_score_from_parts(
                            &st2,
                            finish_time,
                            rounds_val,
                            extra_reps_val,
                            weight_val,
                        );
                        display_score.set(new_score);
                        display_rx.set(rx);
                        editing.set(false);
                        save_result.set(Some(Ok(())));
                    }
                    Err(e) => {
                        save_result.set(Some(Err(e.to_string())));
                    }
                }
            });
        }
    };

    let on_cancel = {
        let orig_minutes = orig_minutes.clone();
        let orig_seconds = orig_seconds.clone();
        let orig_rounds = orig_rounds.clone();
        let orig_extra_reps = orig_extra_reps.clone();
        let orig_weight = orig_weight.clone();
        move |_| {
            minutes.set(orig_minutes.clone());
            seconds.set(orig_seconds.clone());
            rounds.set(orig_rounds.clone());
            extra_reps.set(orig_extra_reps.clone());
            weight.set(orig_weight.clone());
            is_rx.set(orig_rx);
            editing.set(false);
            save_result.set(None);
        }
    };

    view! {
        <div class="result-section-row" class:result-section-row--editing=move || editing.get()>
            <div
                class="result-section-summary"
                on:click=move |_| {
                    if !editing.get() && !s.skipped {
                        editing.set(true);
                    }
                }
            >
                <span class="result-section-label">{label}</span>
                {move || {
                    let rx = display_rx.get();
                    (!s.skipped).then(|| {
                        let cls = if rx { "result-rx" } else { "result-rx result-rx--scaled" };
                        let lbl = if rx { "Rx" } else { "Scaled" };
                        view! { <span class={cls}>{lbl}</span> }
                    })
                }}
                <span class="result-section-score">
                    {move || display_score.get()}
                    {(!s.skipped).then(|| view! {
                        <span class="result-movement-edit-hint">" ✎"</span>
                    })}
                </span>
            </div>
            {move || editing.get().then(|| {
                let fields = match st.as_str() {
                    "fortime" => view! {
                        <div class="section-edit-fields">
                            <div class="movement-edit-field">
                                <label>"Min"</label>
                                <input
                                    type="number"
                                    inputmode="numeric"
                                    min="0"
                                    prop:value=move || minutes.get()
                                    on:input=move |ev| minutes.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="movement-edit-field">
                                <label>"Sec"</label>
                                <input
                                    type="number"
                                    inputmode="numeric"
                                    min="0"
                                    max="59"
                                    prop:value=move || seconds.get()
                                    on:input=move |ev| seconds.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                    }.into_any(),
                    "amrap" | "emom" => view! {
                        <div class="section-edit-fields">
                            <div class="movement-edit-field">
                                <label>"Rounds"</label>
                                <input
                                    type="number"
                                    inputmode="numeric"
                                    min="0"
                                    prop:value=move || rounds.get()
                                    on:input=move |ev| rounds.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="movement-edit-field">
                                <label>"+ Reps"</label>
                                <input
                                    type="number"
                                    inputmode="numeric"
                                    min="0"
                                    prop:value=move || extra_reps.get()
                                    on:input=move |ev| extra_reps.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                    }.into_any(),
                    "strength" => view! {
                        <div class="section-edit-fields">
                            <div class="movement-edit-field">
                                <label>"Weight (kg)"</label>
                                <input
                                    type="number"
                                    inputmode="decimal"
                                    step="0.5"
                                    min="0"
                                    prop:value=move || weight.get()
                                    on:input=move |ev| weight.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                    }.into_any(),
                    _ => view! { <div></div> }.into_any(),
                };
                view! {
                    <div class="section-edit-form">
                        {fields}
                        <div class="section-edit-rx">
                            <label
                                class="rx-toggle"
                                class:rx-toggle--active=move || is_rx.get()
                                on:click=move |_| is_rx.set(!is_rx.get_untracked())
                            >
                                {move || if is_rx.get() { "Rx" } else { "Scaled" }}
                            </label>
                        </div>
                        {move || save_result.get().and_then(|r| r.err()).map(|e| view! {
                            <div class="movement-edit-error">{e}</div>
                        })}
                        <div class="movement-edit-actions">
                            <button
                                class="movement-edit-btn movement-edit-btn--cancel"
                                on:click=on_cancel.clone()
                            >"Cancel"</button>
                            <button
                                class="movement-edit-btn movement-edit-btn--save"
                                disabled=move || saving.get()
                                on:click=on_save.clone()
                            >{move || if saving.get() { "Saving..." } else { "Save" }}</button>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}

/// Per-set signal state for inline editing.
struct SetEditState {
    set_id: String,
    set_number: i32,
    reps: RwSignal<String>,
    weight_kg: RwSignal<String>,
    orig_reps: String,
    orig_weight: String,
}

#[component]
fn EditableMovementRow(m: MovementLogWithName, sets: Vec<MovementLogSet>) -> impl IntoView {
    let editing = RwSignal::new(false);
    let saving = RwSignal::new(false);
    let save_result = RwSignal::new(Option::<Result<(), String>>::None);
    let has_sets = !sets.is_empty();

    let exercise_name = m.exercise_name.clone();
    let movement_log_id = m.id.clone();

    // Display: show per-set detail if available
    let detail = if has_sets {
        let parts: Vec<String> = sets
            .iter()
            .map(|s| {
                let r = s.reps.map(|r| r.to_string()).unwrap_or("-".to_string());
                if let Some(w) = s.weight_kg {
                    format!("{}@{}kg", r, w)
                } else {
                    r
                }
            })
            .collect();
        parts.join(" / ")
    } else {
        format_movement(&m)
    };
    let display_detail = RwSignal::new(detail);

    // Build per-set edit states
    let set_states: Vec<SetEditState> = sets
        .iter()
        .map(|s| {
            let r = s.reps.map(|r| r.to_string()).unwrap_or_default();
            let w = s.weight_kg.map(|w| w.to_string()).unwrap_or_default();
            SetEditState {
                set_id: s.id.clone(),
                set_number: s.set_number,
                reps: RwSignal::new(r.clone()),
                weight_kg: RwSignal::new(w.clone()),
                orig_reps: r,
                orig_weight: w,
            }
        })
        .collect();

    // Flat (aggregated) edit state for movements without per-set data
    let reps_val = RwSignal::new(m.reps.map(|r| r.to_string()).unwrap_or_default());
    let sets_val = RwSignal::new(m.sets.map(|s| s.to_string()).unwrap_or_default());
    let weight_val = RwSignal::new(m.weight_kg.map(|w| w.to_string()).unwrap_or_default());
    let notes_val = RwSignal::new(m.notes.clone().unwrap_or_default());
    let orig_reps = m.reps.map(|r| r.to_string()).unwrap_or_default();
    let orig_sets = m.sets.map(|s| s.to_string()).unwrap_or_default();
    let orig_weight = m.weight_kg.map(|w| w.to_string()).unwrap_or_default();
    let orig_notes = m.notes.clone().unwrap_or_default();

    // Extract (set_id, reps_signal, weight_signal) for the save closure
    let set_save_data: Vec<(String, RwSignal<String>, RwSignal<String>)> = set_states
        .iter()
        .map(|s| (s.set_id.clone(), s.reps, s.weight_kg))
        .collect();

    let on_save = move |_| {
        saving.set(true);
        save_result.set(None);

        if has_sets {
            // Save each set individually, then update the aggregate on the parent movement_log
            let futures: Vec<_> = set_save_data
                .iter()
                .map(|(sid, reps_sig, weight_sig)| {
                    let sid = sid.clone();
                    let r: Option<i32> = reps_sig.get_untracked().parse().ok();
                    let w: Option<f32> = weight_sig.get_untracked().parse().ok();
                    (sid, r, w)
                })
                .collect();

            // Compute new aggregated values
            let total_reps: i32 = futures.iter().filter_map(|(_, r, _)| *r).sum();
            let num_sets = futures.len() as i32;
            let max_weight: Option<f32> =
                futures.iter().filter_map(|(_, _, w)| *w).reduce(f32::max);

            // Build new display string
            let parts: Vec<String> = futures
                .iter()
                .map(|(_, r, w)| {
                    let rs = r.map(|r| r.to_string()).unwrap_or("-".to_string());
                    if let Some(wv) = w {
                        format!("{}@{}kg", rs, wv)
                    } else {
                        rs
                    }
                })
                .collect();
            let new_detail = parts.join(" / ");

            let ml_id = movement_log_id.clone();
            let notes_str = notes_val.get_untracked();
            let notes = if notes_str.is_empty() {
                None
            } else {
                Some(notes_str)
            };

            leptos::task::spawn_local(async move {
                // Update all sets concurrently
                let mut any_err = None;
                for (sid, r, w) in &futures {
                    if let Err(e) = update_movement_log_set(sid.clone(), *r, *w).await {
                        any_err = Some(e.to_string());
                        break;
                    }
                }
                // Also update the aggregate movement_log row
                if any_err.is_none() {
                    let agg_reps = if total_reps > 0 {
                        Some(total_reps)
                    } else {
                        None
                    };
                    if let Err(e) =
                        update_movement_log(ml_id, agg_reps, Some(num_sets), max_weight, notes)
                            .await
                    {
                        any_err = Some(e.to_string());
                    }
                }

                saving.set(false);
                match any_err {
                    None => {
                        display_detail.set(new_detail);
                        editing.set(false);
                        save_result.set(Some(Ok(())));
                    }
                    Some(e) => {
                        save_result.set(Some(Err(e)));
                    }
                }
            });
        } else {
            // Flat save (no per-set data)
            let ml_id = movement_log_id.clone();
            let reps: Option<i32> = reps_val.get_untracked().parse().ok();
            let sets: Option<i32> = sets_val.get_untracked().parse().ok();
            let weight: Option<f32> = weight_val.get_untracked().parse().ok();
            let notes_str = notes_val.get_untracked();
            let notes = if notes_str.is_empty() {
                None
            } else {
                Some(notes_str)
            };
            let new_detail = format_movement_parts(reps, sets, weight);

            leptos::task::spawn_local(async move {
                let result = update_movement_log(ml_id, reps, sets, weight, notes).await;
                saving.set(false);
                match result {
                    Ok(_) => {
                        display_detail.set(new_detail);
                        editing.set(false);
                        save_result.set(Some(Ok(())));
                    }
                    Err(e) => {
                        save_result.set(Some(Err(e.to_string())));
                    }
                }
            });
        }
    };

    // Extract restore data for cancel (signal + original value)
    let set_restore: Vec<(RwSignal<String>, String, RwSignal<String>, String)> = set_states
        .iter()
        .map(|s| {
            (
                s.reps,
                s.orig_reps.clone(),
                s.weight_kg,
                s.orig_weight.clone(),
            )
        })
        .collect();

    let on_cancel = {
        let orig_reps = orig_reps.clone();
        let orig_sets = orig_sets.clone();
        let orig_weight = orig_weight.clone();
        let orig_notes = orig_notes.clone();
        move |_| {
            // Restore flat values
            reps_val.set(orig_reps.clone());
            sets_val.set(orig_sets.clone());
            weight_val.set(orig_weight.clone());
            notes_val.set(orig_notes.clone());
            // Restore per-set values
            for (reps_sig, orig_r, weight_sig, orig_w) in &set_restore {
                reps_sig.set(orig_r.clone());
                weight_sig.set(orig_w.clone());
            }
            editing.set(false);
            save_result.set(None);
        }
    };

    // Store set signal pairs for building views inside reactive closures
    let set_signals: Vec<(i32, RwSignal<String>, RwSignal<String>)> = set_states
        .iter()
        .map(|s| (s.set_number, s.reps, s.weight_kg))
        .collect();

    view! {
        <div class="result-movement-row" class:result-movement-row--editing=move || editing.get()>
            <div
                class="result-movement-summary"
                on:click=move |_| {
                    if !editing.get() {
                        editing.set(true);
                    }
                }
            >
                <span class="result-movement-name">{exercise_name}</span>
                <span class="result-movement-detail">
                    {move || display_detail.get()}
                    <span class="result-movement-edit-hint">" ✎"</span>
                </span>
            </div>
            {move || editing.get().then(|| {
                if has_sets {
                    // Per-set edit form
                    view! {
                        <div class="movement-edit-form">
                            <div class="movement-set-edit-rows">
                                {set_signals.iter().map(|(num, reps_sig, weight_sig)| {
                                    let label = format!("Set {}", num);
                                    let r = *reps_sig;
                                    let w = *weight_sig;
                                    view! {
                                        <div class="movement-set-edit-row">
                                            <span class="movement-set-label">{label}</span>
                                            <div class="movement-edit-field">
                                                <input
                                                    type="number"
                                                    inputmode="numeric"
                                                    min="0"
                                                    placeholder="reps"
                                                    prop:value=move || r.get()
                                                    on:input=move |ev| r.set(event_target_value(&ev))
                                                />
                                            </div>
                                            <div class="movement-edit-field">
                                                <input
                                                    type="number"
                                                    inputmode="decimal"
                                                    step="0.5"
                                                    min="0"
                                                    placeholder="kg"
                                                    prop:value=move || w.get()
                                                    on:input=move |ev| w.set(event_target_value(&ev))
                                                />
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                            {move || save_result.get().and_then(|r| r.err()).map(|e| view! {
                                <div class="movement-edit-error">{e}</div>
                            })}
                            <div class="movement-edit-actions">
                                <button
                                    class="movement-edit-btn movement-edit-btn--cancel"
                                    on:click=on_cancel.clone()
                                >"Cancel"</button>
                                <button
                                    class="movement-edit-btn movement-edit-btn--save"
                                    class:btn--loading=move || saving.get()
                                    disabled=move || saving.get()
                                    on:click=on_save.clone()
                                >{move || if saving.get() { "Saving..." } else { "Save" }}</button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    // Flat edit form
                    view! {
                        <div class="movement-edit-form">
                            <div class="movement-edit-fields">
                                <div class="movement-edit-field">
                                    <label>"Reps"</label>
                                    <input
                                        type="number"
                                        inputmode="numeric"
                                        min="0"
                                        prop:value=move || reps_val.get()
                                        on:input=move |ev| reps_val.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="movement-edit-field">
                                    <label>"Sets"</label>
                                    <input
                                        type="number"
                                        inputmode="numeric"
                                        min="0"
                                        prop:value=move || sets_val.get()
                                        on:input=move |ev| sets_val.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="movement-edit-field">
                                    <label>"Weight (kg)"</label>
                                    <input
                                        type="number"
                                        inputmode="decimal"
                                        step="0.5"
                                        min="0"
                                        prop:value=move || weight_val.get()
                                        on:input=move |ev| weight_val.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>
                            <div class="movement-edit-field movement-edit-field--notes">
                                <label>"Notes"</label>
                                <input
                                    type="text"
                                    placeholder="notes..."
                                    prop:value=move || notes_val.get()
                                    on:input=move |ev| notes_val.set(event_target_value(&ev))
                                />
                            </div>
                            {move || save_result.get().and_then(|r| r.err()).map(|e| view! {
                                <div class="movement-edit-error">{e}</div>
                            })}
                            <div class="movement-edit-actions">
                                <button
                                    class="movement-edit-btn movement-edit-btn--cancel"
                                    on:click=on_cancel.clone()
                                >"Cancel"</button>
                                <button
                                    class="movement-edit-btn movement-edit-btn--save"
                                    class:btn--loading=move || saving.get()
                                    disabled=move || saving.get()
                                    on:click=on_save.clone()
                                >{move || if saving.get() { "Saving..." } else { "Save" }}</button>
                            </div>
                        </div>
                    }.into_any()
                }
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

fn format_movement_parts(reps: Option<i32>, sets: Option<i32>, weight_kg: Option<f32>) -> String {
    let mut parts = Vec::new();
    if let Some(s) = sets {
        if s > 1 {
            parts.push(format!("{}×", s));
        }
    }
    if let Some(r) = reps {
        parts.push(format!("{} reps", r));
    }
    if let Some(w) = weight_kg {
        parts.push(format!("{}kg", w));
    }
    if parts.is_empty() {
        "—".to_string()
    } else {
        parts.join(" ")
    }
}

fn format_movement(m: &MovementLogWithName) -> String {
    format_movement_parts(m.reps, m.sets, m.weight_kg)
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

fn format_score_from_parts(
    section_type: &str,
    finish_time: Option<i32>,
    rounds_completed: Option<i32>,
    extra_reps: Option<i32>,
    weight_kg: Option<f32>,
) -> String {
    match section_type {
        "fortime" => {
            if let Some(t) = finish_time {
                format!("{}:{:02}", t / 60, t % 60)
            } else {
                "—".to_string()
            }
        }
        "amrap" | "emom" => {
            let rounds = rounds_completed.unwrap_or(0);
            let reps = extra_reps.unwrap_or(0);
            if reps > 0 {
                format!("{} rounds + {} reps", rounds, reps)
            } else {
                format!("{} rounds", rounds)
            }
        }
        "strength" => {
            if let Some(w) = weight_kg {
                format!("{}kg", w)
            } else {
                "—".to_string()
            }
        }
        _ => "—".to_string(),
    }
}

fn format_section_score(s: &SectionScoreWithMeta) -> String {
    format_score_from_parts(
        &s.section_type,
        s.finish_time_seconds,
        s.rounds_completed,
        s.extra_reps,
        s.weight_kg,
    )
}
