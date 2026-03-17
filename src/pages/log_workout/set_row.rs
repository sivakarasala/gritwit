use leptos::prelude::*;

use super::custom_log::{ExerciseEntry, SetData};

/// A single set row: inputs vary based on scoring_type.
#[component]
pub(super) fn SetRow(
    entry_key: usize,
    set: SetData,
    scoring_type: String,
    exercises: RwSignal<Vec<ExerciseEntry>>,
) -> impl IntoView {
    let sn = set.set_number;
    let set_label = format!("Set {}", sn);

    let reps = RwSignal::new(set.reps);
    let weight_kg = RwSignal::new(set.weight_kg);
    let duration = RwSignal::new(set.duration);
    let distance_meters = RwSignal::new(set.distance_meters);
    let calories = RwSignal::new(set.calories);
    let notes = RwSignal::new(set.notes);
    let show_weight = RwSignal::new(set.show_weight);
    let show_notes = RwSignal::new(set.show_notes);

    let sync_field =
        move |signal: RwSignal<String>, field_fn: fn(&mut SetData) -> &mut String, val: String| {
            signal.set(val.clone());
            exercises.update_untracked(|list| {
                if let Some(e) = list.iter_mut().find(|e| e.key == entry_key) {
                    if let Some(s) = e.sets.iter_mut().find(|s| s.set_number == sn) {
                        *field_fn(s) = val;
                    }
                }
            });
        };

    let sync_reps = move |val: String| sync_field(reps, |s| &mut s.reps, val);
    let sync_weight = move |val: String| sync_field(weight_kg, |s| &mut s.weight_kg, val);
    let sync_duration = move |val: String| sync_field(duration, |s| &mut s.duration, val);
    let sync_distance =
        move |val: String| sync_field(distance_meters, |s| &mut s.distance_meters, val);
    let sync_calories = move |val: String| sync_field(calories, |s| &mut s.calories, val);
    let sync_notes = move |val: String| sync_field(notes, |s| &mut s.notes, val);

    let toggle_weight = move |_| {
        show_weight.set(true);
        exercises.update_untracked(|list| {
            if let Some(e) = list.iter_mut().find(|e| e.key == entry_key) {
                if let Some(s) = e.sets.iter_mut().find(|s| s.set_number == sn) {
                    s.show_weight = true;
                }
            }
        });
    };

    let toggle_notes = move |_| {
        show_notes.set(true);
        exercises.update_untracked(|list| {
            if let Some(e) = list.iter_mut().find(|e| e.key == entry_key) {
                if let Some(s) = e.sets.iter_mut().find(|s| s.set_number == sn) {
                    s.show_notes = true;
                }
            }
        });
    };

    let st = StoredValue::new(scoring_type);

    view! {
        <div class="set-row">
            <span class="set-number">{set_label}</span>
            <div class="set-inputs">
                {move || match st.get_value().as_str() {
                    "distance" => view! {
                        <div class="set-field">
                            <input
                                type="number"
                                class="score-input set-input"
                                placeholder="meters"
                                inputmode="decimal"
                                step="1"
                                min="0"
                                prop:value=move || distance_meters.get()
                                on:input=move |ev| sync_distance(event_target_value(&ev))
                            />
                            <span class="set-unit">"m"</span>
                        </div>
                    }.into_any(),
                    "calories" => view! {
                        <div class="set-field">
                            <input
                                type="number"
                                class="score-input set-input"
                                placeholder="cal"
                                inputmode="numeric"
                                step="1"
                                min="0"
                                prop:value=move || calories.get()
                                on:input=move |ev| sync_calories(event_target_value(&ev))
                            />
                            <span class="set-unit">"cal"</span>
                        </div>
                    }.into_any(),
                    "time" => view! {
                        <div class="set-field">
                            <input
                                type="number"
                                class="score-input set-input"
                                placeholder="sec"
                                inputmode="numeric"
                                step="1"
                                min="0"
                                prop:value=move || duration.get()
                                on:input=move |ev| sync_duration(event_target_value(&ev))
                            />
                            <span class="set-unit">"s"</span>
                        </div>
                    }.into_any(),
                    "reps_only" => view! {
                        <div class="set-field">
                            <input
                                type="number"
                                class="score-input set-input"
                                placeholder="reps"
                                inputmode="numeric"
                                min="0"
                                prop:value=move || reps.get()
                                on:input=move |ev| sync_reps(event_target_value(&ev))
                            />
                        </div>
                    }.into_any(),
                    // weight_and_reps (default)
                    _ => view! {
                        <div class="set-field">
                            <input
                                type="number"
                                class="score-input set-input"
                                placeholder="reps"
                                inputmode="numeric"
                                min="0"
                                prop:value=move || reps.get()
                                on:input=move |ev| sync_reps(event_target_value(&ev))
                            />
                        </div>
                        {move || {
                            if show_weight.get() {
                                view! {
                                    <div class="set-field set-field--weight">
                                        <input
                                            type="number"
                                            class="score-input set-input"
                                            placeholder="kg"
                                            inputmode="decimal"
                                            step="0.5"
                                            min="0"
                                            prop:value=move || weight_kg.get()
                                            on:input=move |ev| sync_weight(event_target_value(&ev))
                                        />
                                        <span class="set-unit">"kg"</span>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <button class="set-toggle-btn" on:click=toggle_weight>
                                        "+ weight"
                                    </button>
                                }.into_any()
                            }
                        }}
                    }.into_any(),
                }}
            </div>

            {move || {
                if show_notes.get() {
                    view! {
                        <input
                            type="text"
                            class="set-notes-input"
                            placeholder="notes..."
                            prop:value=move || notes.get()
                            on:input=move |ev| sync_notes(event_target_value(&ev))
                        />
                    }.into_any()
                } else {
                    view! {
                        <button
                            class="set-toggle-btn set-toggle-btn--notes"
                            on:click=toggle_notes
                        >
                            "+ notes"
                        </button>
                    }.into_any()
                }
            }}
        </div>
    }
}
