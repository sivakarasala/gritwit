use leptos::prelude::*;

use super::custom_log::{ExerciseEntry, SetData};

/// A single set row: reps input + optional weight/notes toggles.
#[component]
pub(super) fn SetRow(
    entry_key: usize,
    set: SetData,
    exercises: RwSignal<Vec<ExerciseEntry>>,
) -> impl IntoView {
    let sn = set.set_number;
    let set_label = format!("Set {}", sn);

    let reps = RwSignal::new(set.reps);
    let weight_kg = RwSignal::new(set.weight_kg);
    let _duration = RwSignal::new(set.duration);
    let notes = RwSignal::new(set.notes);
    let show_weight = RwSignal::new(set.show_weight);
    let show_notes = RwSignal::new(set.show_notes);

    let sync_reps = move |val: String| {
        reps.set(val.clone());
        exercises.update_untracked(|list| {
            if let Some(e) = list.iter_mut().find(|e| e.key == entry_key) {
                if let Some(s) = e.sets.iter_mut().find(|s| s.set_number == sn) {
                    s.reps = val;
                }
            }
        });
    };

    let sync_weight = move |val: String| {
        weight_kg.set(val.clone());
        exercises.update_untracked(|list| {
            if let Some(e) = list.iter_mut().find(|e| e.key == entry_key) {
                if let Some(s) = e.sets.iter_mut().find(|s| s.set_number == sn) {
                    s.weight_kg = val;
                }
            }
        });
    };

    let sync_notes = move |val: String| {
        notes.set(val.clone());
        exercises.update_untracked(|list| {
            if let Some(e) = list.iter_mut().find(|e| e.key == entry_key) {
                if let Some(s) = e.sets.iter_mut().find(|s| s.set_number == sn) {
                    s.notes = val;
                }
            }
        });
    };

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

    view! {
        <div class="set-row">
            <span class="set-number">{set_label}</span>
            <div class="set-inputs">
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
                        }
                        .into_any()
                    } else {
                        view! {
                            <button
                                class="set-toggle-btn"
                                on:click=toggle_weight
                            >
                                "+ weight"
                            </button>
                        }
                        .into_any()
                    }
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
                    }
                    .into_any()
                } else {
                    view! {
                        <button
                            class="set-toggle-btn set-toggle-btn--notes"
                            on:click=toggle_notes
                        >
                            "+ notes"
                        </button>
                    }
                    .into_any()
                }
            }}
        </div>
    }
}
