use crate::db::{MovementLogSet, MovementLogWithName};
use leptos::prelude::*;

use super::{update_movement_log, update_movement_log_set};

type SetSaveEntry = (
    String,
    RwSignal<String>,
    RwSignal<String>,
    RwSignal<String>,
    RwSignal<String>,
);
type SetRestoreEntry = (
    RwSignal<String>,
    String,
    RwSignal<String>,
    String,
    RwSignal<String>,
    String,
    RwSignal<String>,
    String,
);
type SetSignalEntry = (
    i32,
    RwSignal<String>,
    RwSignal<String>,
    RwSignal<String>,
    RwSignal<String>,
);

/// Per-set signal state for inline editing.
struct SetEditState {
    set_id: String,
    set_number: i32,
    reps: RwSignal<String>,
    weight_kg: RwSignal<String>,
    distance_meters: RwSignal<String>,
    calories: RwSignal<String>,
    orig_reps: String,
    orig_weight: String,
    orig_distance: String,
    orig_calories: String,
}

#[component]
pub(super) fn EditableMovementRow(
    m: MovementLogWithName,
    sets: Vec<MovementLogSet>,
) -> impl IntoView {
    let editing = RwSignal::new(false);
    let saving = RwSignal::new(false);
    let save_result = RwSignal::new(Option::<Result<(), String>>::None);
    let has_sets = !sets.is_empty();

    let exercise_name = m.exercise_name.clone();
    let movement_log_id = m.id.clone();

    // Display: show per-set detail if available
    let scoring_type = m.scoring_type.clone();
    let detail = if has_sets {
        let parts: Vec<String> = sets
            .iter()
            .map(|s| format_set_log(s, &scoring_type))
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
            let d = s.distance_meters.map(|v| v.to_string()).unwrap_or_default();
            let c = s.calories.map(|v| v.to_string()).unwrap_or_default();
            SetEditState {
                set_id: s.id.clone(),
                set_number: s.set_number,
                reps: RwSignal::new(r.clone()),
                weight_kg: RwSignal::new(w.clone()),
                distance_meters: RwSignal::new(d.clone()),
                calories: RwSignal::new(c.clone()),
                orig_reps: r,
                orig_weight: w,
                orig_distance: d,
                orig_calories: c,
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

    let scoring_type_sv = StoredValue::new(scoring_type.clone());

    // Extract (set_id, reps, weight, distance_meters, calories) for save closure
    let set_save_data: Vec<SetSaveEntry> = set_states
        .iter()
        .map(|s| {
            (
                s.set_id.clone(),
                s.reps,
                s.weight_kg,
                s.distance_meters,
                s.calories,
            )
        })
        .collect();

    let on_save = move |_| {
        saving.set(true);
        save_result.set(None);

        if has_sets {
            // Save each set individually, then update the aggregate on the parent movement_log
            let st = scoring_type_sv.get_value();
            let set_inputs: Vec<_> = set_save_data
                .iter()
                .map(|(sid, reps_sig, weight_sig, dist_sig, cal_sig)| {
                    let sid = sid.clone();
                    let r: Option<i32> = reps_sig.get_untracked().parse().ok();
                    let w: Option<f32> = weight_sig.get_untracked().parse().ok();
                    let d: Option<f32> = dist_sig.get_untracked().parse().ok();
                    let c: Option<i32> = cal_sig.get_untracked().parse().ok();
                    (sid, r, w, d, c)
                })
                .collect();

            // Compute new aggregated values
            let total_reps: i32 = set_inputs.iter().filter_map(|(_, r, _, _, _)| *r).sum();
            let num_sets = set_inputs.len() as i32;
            let max_weight: Option<f32> = set_inputs
                .iter()
                .filter_map(|(_, _, w, _, _)| *w)
                .reduce(f32::max);

            // Build new display string based on scoring type
            let parts: Vec<String> = set_inputs
                .iter()
                .map(|(_, r, w, d, c)| match st.as_str() {
                    "distance" => d.map(|v| format!("{}m", v)).unwrap_or("-".to_string()),
                    "calories" => c.map(|v| format!("{} cal", v)).unwrap_or("-".to_string()),
                    _ => {
                        let rs = r.map(|r| r.to_string()).unwrap_or("-".to_string());
                        if let Some(wv) = w {
                            format!("{}@{}kg", rs, wv)
                        } else {
                            rs
                        }
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
                let mut any_err = None;
                for (sid, r, w, d, c) in &set_inputs {
                    if let Err(e) = update_movement_log_set(sid.clone(), *r, *w, *d, *c).await {
                        any_err = Some(e.to_string());
                        break;
                    }
                }
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

    // Extract restore data for cancel
    let set_restore: Vec<SetRestoreEntry> = set_states
        .iter()
        .map(|s| {
            (
                s.reps,
                s.orig_reps.clone(),
                s.weight_kg,
                s.orig_weight.clone(),
                s.distance_meters,
                s.orig_distance.clone(),
                s.calories,
                s.orig_calories.clone(),
            )
        })
        .collect();

    let on_cancel = {
        let orig_reps = orig_reps.clone();
        let orig_sets = orig_sets.clone();
        let orig_weight = orig_weight.clone();
        let orig_notes = orig_notes.clone();
        move |_| {
            reps_val.set(orig_reps.clone());
            sets_val.set(orig_sets.clone());
            weight_val.set(orig_weight.clone());
            notes_val.set(orig_notes.clone());
            for (reps_sig, orig_r, weight_sig, orig_w, dist_sig, orig_d, cal_sig, orig_c) in
                &set_restore
            {
                reps_sig.set(orig_r.clone());
                weight_sig.set(orig_w.clone());
                dist_sig.set(orig_d.clone());
                cal_sig.set(orig_c.clone());
            }
            editing.set(false);
            save_result.set(None);
        }
    };

    // Store set signals for building views inside reactive closures
    let set_signals: Vec<SetSignalEntry> = set_states
        .iter()
        .map(|s| {
            (
                s.set_number,
                s.reps,
                s.weight_kg,
                s.distance_meters,
                s.calories,
            )
        })
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
                    let st = scoring_type_sv.get_value();
                    view! {
                        <div class="movement-edit-form">
                            <div class="movement-set-edit-rows">
                                {set_signals.iter().map(|(num, reps_sig, weight_sig, dist_sig, cal_sig)| {
                                    let label = format!("Set {}", num);
                                    let r = *reps_sig;
                                    let w = *weight_sig;
                                    let d = *dist_sig;
                                    let c = *cal_sig;
                                    let st2 = st.clone();
                                    view! {
                                        <div class="movement-set-edit-row">
                                            <span class="movement-set-label">{label}</span>
                                            {match st2.as_str() {
                                                "distance" => view! {
                                                    <div class="movement-edit-field">
                                                        <input
                                                            type="number"
                                                            inputmode="decimal"
                                                            step="1"
                                                            min="0"
                                                            placeholder="m"
                                                            prop:value=move || d.get()
                                                            on:input=move |ev| d.set(event_target_value(&ev))
                                                        />
                                                    </div>
                                                }.into_any(),
                                                "calories" => view! {
                                                    <div class="movement-edit-field">
                                                        <input
                                                            type="number"
                                                            inputmode="numeric"
                                                            step="1"
                                                            min="0"
                                                            placeholder="cal"
                                                            prop:value=move || c.get()
                                                            on:input=move |ev| c.set(event_target_value(&ev))
                                                        />
                                                    </div>
                                                }.into_any(),
                                                _ => view! {
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
                                                }.into_any(),
                                            }}
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

/// Format a per-set movement log entry for history display.
fn format_set_log(set: &MovementLogSet, scoring_type: &str) -> String {
    match scoring_type {
        "distance" => set
            .distance_meters
            .map(|v| format!("{}m", v))
            .unwrap_or_else(|| "-".to_string()),
        "calories" => set
            .calories
            .map(|v| format!("{} cal", v))
            .unwrap_or_else(|| "-".to_string()),
        _ => {
            let r = set.reps.map(|r| r.to_string()).unwrap_or("-".to_string());
            if let Some(w) = set.weight_kg {
                format!("{}@{}kg", r, w)
            } else {
                r
            }
        }
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
