use crate::db::ExerciseSetInput;
use leptos::prelude::*;

use super::exercise_entry_card::ExerciseEntryCard;

// ---- Internal types (plain data, no nested signals) ----

#[derive(Clone)]
pub(super) struct ExerciseEntry {
    pub key: usize,
    pub exercise_id: String,
    pub exercise_name: String,
    pub sets: Vec<SetData>,
}

#[derive(Clone)]
pub(super) struct SetData {
    pub set_number: i32,
    pub reps: String,
    pub weight_kg: String,
    pub duration: String,
    pub notes: String,
    pub show_weight: bool,
    pub show_notes: bool,
}

impl SetData {
    pub fn new(num: i32) -> Self {
        Self {
            set_number: num,
            reps: String::new(),
            weight_kg: String::new(),
            duration: String::new(),
            notes: String::new(),
            show_weight: false,
            show_notes: false,
        }
    }
}

/// Custom workout log (no WOD attached). Supports both create and edit modes.
#[component]
pub fn CustomLogFlow(edit_id: String) -> impl IntoView {
    let is_edit = !edit_id.is_empty();
    let edit_id_signal = RwSignal::new(edit_id.clone());

    let today = crate::pages::wod::week_calendar::today_iso();
    let workout_date = RwSignal::new(today);
    let notes = RwSignal::new(String::new());
    let submit_result = RwSignal::new(Option::<Result<String, String>>::None);
    let submitting = RwSignal::new(false);

    let exercises = RwSignal::new(Vec::<ExerciseEntry>::new());
    let next_key = RwSignal::new(0usize);

    // Load existing workout data when editing
    let edit_data = Resource::new(
        move || edit_id.clone(),
        |id| async move {
            if id.is_empty() {
                return Ok(None);
            }
            super::server_fns::get_workout_for_edit(id).await.map(Some)
        },
    );

    // Pre-fill form when edit data loads
    Effect::new(move || {
        if let Some(Ok(Some((log, exs)))) = edit_data.get() {
            workout_date.set(log.workout_date);
            notes.set(log.notes.unwrap_or_default());

            let mut entries: Vec<ExerciseEntry> = Vec::new();
            let mut key_counter = 0usize;
            for ex in &exs {
                if let Some(entry) = entries.iter_mut().find(|e| e.exercise_id == ex.exercise_id) {
                    entry.sets.push(SetData {
                        set_number: ex.set_number,
                        reps: ex.reps.map(|v| v.to_string()).unwrap_or_default(),
                        weight_kg: ex.weight_kg.map(|v| v.to_string()).unwrap_or_default(),
                        duration: ex
                            .duration_seconds
                            .map(|v| v.to_string())
                            .unwrap_or_default(),
                        notes: ex.notes.clone().unwrap_or_default(),
                        show_weight: ex.weight_kg.is_some(),
                        show_notes: ex.notes.is_some(),
                    });
                } else {
                    entries.push(ExerciseEntry {
                        key: key_counter,
                        exercise_id: ex.exercise_id.clone(),
                        exercise_name: ex.exercise_name.clone(),
                        sets: vec![SetData {
                            set_number: ex.set_number,
                            reps: ex.reps.map(|v| v.to_string()).unwrap_or_default(),
                            weight_kg: ex.weight_kg.map(|v| v.to_string()).unwrap_or_default(),
                            duration: ex
                                .duration_seconds
                                .map(|v| v.to_string())
                                .unwrap_or_default(),
                            notes: ex.notes.clone().unwrap_or_default(),
                            show_weight: ex.weight_kg.is_some(),
                            show_notes: ex.notes.is_some(),
                        }],
                    });
                    key_counter += 1;
                }
            }
            next_key.set(key_counter);
            exercises.set(entries);
        }
    });

    // Load exercise list for picker
    let exercise_list = Resource::new(|| (), |_| super::server_fns::list_exercises_for_picker());

    // Picker state
    let picker_open = RwSignal::new(false);
    let picker_search = RwSignal::new(String::new());

    let on_submit = move |_| {
        let date = workout_date.get_untracked();
        if date.is_empty() {
            submit_result.set(Some(Err("Please select a date".to_string())));
            return;
        }

        let entries = exercises.get_untracked();
        let sets: Vec<ExerciseSetInput> = entries
            .iter()
            .flat_map(|entry| {
                entry.sets.iter().map(|set| ExerciseSetInput {
                    exercise_id: entry.exercise_id.clone(),
                    set_number: set.set_number,
                    reps: set.reps.parse().ok(),
                    weight_kg: set.weight_kg.parse().ok(),
                    duration_seconds: set.duration.parse().ok(),
                    notes: if set.notes.is_empty() {
                        None
                    } else {
                        Some(set.notes.clone())
                    },
                })
            })
            .collect();

        if sets.is_empty() {
            submit_result.set(Some(Err("Add at least one exercise".to_string())));
            return;
        }

        // Every exercise must have at least one set with reps
        for entry in &entries {
            let has_data = entry
                .sets
                .iter()
                .any(|s| !s.reps.is_empty() || !s.weight_kg.is_empty() || !s.duration.is_empty());
            if !has_data {
                submit_result.set(Some(Err(format!(
                    "Fill in at least one set for {}",
                    entry.exercise_name
                ))));
                return;
            }
        }

        let exercises_json = serde_json::to_string(&sets).unwrap_or_default();
        let notes_val = notes.get_untracked();
        let log_id = edit_id_signal.get_untracked();

        submitting.set(true);
        submit_result.set(None);

        leptos::task::spawn_local(async move {
            let nav_date = date.clone();
            let result = if log_id.is_empty() {
                super::server_fns::submit_custom_workout(date, notes_val, exercises_json)
                    .await
                    .map(|_| ())
            } else {
                super::server_fns::update_custom_workout(log_id, date, notes_val, exercises_json)
                    .await
            };
            submitting.set(false);
            match result {
                Ok(_) => {
                    exercises.set(Vec::new());
                    notes.set(String::new());
                    let msg = if is_edit {
                        "Workout updated!"
                    } else {
                        "Workout logged!"
                    };
                    submit_result.set(Some(Ok(msg.to_string())));
                    let navigate = leptos_router::hooks::use_navigate();
                    set_timeout(
                        move || {
                            navigate(&format!("/history?date={}", nav_date), Default::default())
                        },
                        std::time::Duration::from_millis(800),
                    );
                }
                Err(e) => {
                    let raw = e.to_string();
                    let clean = raw
                        .strip_prefix("error running server function: ")
                        .or_else(|| raw.strip_prefix("ServerFnError: "))
                        .unwrap_or(&raw);
                    submit_result.set(Some(Err(clean.to_string())));
                }
            }
        });
    };

    let heading = if is_edit { "Edit Workout" } else { "" };
    let submit_label = if is_edit {
        "Update Workout"
    } else {
        "Log Workout"
    };
    let submitting_label = if is_edit { "Updating..." } else { "Logging..." };

    view! {
        <div class="custom-log-form">
            {(!heading.is_empty()).then(|| view! {
                <h2 class="custom-log-heading">{heading}</h2>
            })}

            <div class="score-field">
                <label class="score-label">"Date"</label>
                <input
                    type="date"
                    class="score-input date-input"
                    prop:value=move || workout_date.get()
                    on:input=move |ev| workout_date.set(event_target_value(&ev))
                />
            </div>

            <div class="custom-exercises">
                <div class="custom-exercises-header">
                    <span class="score-label">"Exercises"</span>
                </div>

                <div class="exercise-list-region">
                    {move || {
                        let entries = exercises.get();
                        if entries.is_empty() {
                            view! {
                                <p class="custom-empty-hint">"Tap + to add an exercise"</p>
                            }
                            .into_any()
                        } else {
                            view! {
                                <div class="exercise-cards">
                                    {entries
                                        .into_iter()
                                        .map(|entry| {
                                            let key = entry.key;
                                            view! {
                                                <ExerciseEntryCard
                                                    entry_key=key
                                                    exercise_name=entry.exercise_name.clone()
                                                    sets=entry.sets.clone()
                                                    exercises=exercises
                                                />
                                            }
                                        })
                                        .collect_view()}
                                </div>
                            }
                            .into_any()
                        }
                    }}
                </div>

                <div class="exercise-picker-region">
                    <Suspense fallback=|| ()>
                        {move || {
                            let available = exercise_list
                                .get()
                                .and_then(|r| r.ok())
                                .unwrap_or_default();

                            if picker_open.get() {
                                let q = picker_search.get().to_lowercase();
                                let filtered: Vec<_> = available
                                    .iter()
                                    .filter(|ex| q.is_empty() || ex.name.to_lowercase().contains(&q))
                                    .cloned()
                                    .collect();

                                view! {
                                    <div class="exercise-picker-panel">
                                        <input
                                            type="text"
                                            class="exercise-picker-search"
                                            placeholder="Search exercises..."
                                            prop:value=move || picker_search.get()
                                            on:input=move |ev| picker_search.set(event_target_value(&ev))
                                        />
                                        <div class="exercise-picker-list">
                                            {filtered
                                                .into_iter()
                                                .map(|ex| {
                                                    let ex_id = ex.id.clone();
                                                    let ex_name = ex.name.clone();
                                                    let display_name = ex.name.clone();
                                                    let ex_cat = ex.category.clone();
                                                    view! {
                                                        <button
                                                            class="exercise-picker-item"
                                                            on:click=move |_| {
                                                                let k = next_key.get_untracked();
                                                                next_key.set(k + 1);
                                                                exercises.update(|list| {
                                                                    list.push(ExerciseEntry {
                                                                        key: k,
                                                                        exercise_id: ex_id.clone(),
                                                                        exercise_name: ex_name.clone(),
                                                                        sets: vec![SetData::new(1)],
                                                                    });
                                                                });
                                                                picker_open.set(false);
                                                                picker_search.set(String::new());
                                                            }
                                                        >
                                                            <span class="picker-item-name">{display_name}</span>
                                                            <span class="picker-item-cat">{ex_cat}</span>
                                                        </button>
                                                    }
                                                })
                                                .collect_view()}
                                        </div>
                                        <button
                                            class="exercise-picker-close"
                                            on:click=move |_| {
                                                picker_open.set(false);
                                                picker_search.set(String::new());
                                            }
                                        >
                                            "Cancel"
                                        </button>
                                    </div>
                                }
                                .into_any()
                            } else {
                                view! {
                                    <button class="add-exercise-btn" on:click=move |_| picker_open.set(true)>
                                        "+ Add Exercise"
                                    </button>
                                }
                                .into_any()
                            }
                        }}
                    </Suspense>
                </div>
            </div>

            <div class="score-field">
                <label class="score-label">"Notes (optional)"</label>
                <textarea
                    class="score-textarea"
                    placeholder="How did it feel?"
                    prop:value=move || notes.get()
                    on:input=move |ev| notes.set(event_target_value(&ev))
                ></textarea>
            </div>

            {move || {
                submit_result.get().map(|r| match r {
                    Ok(msg) => view! { <div class="score-success">{msg}</div> }.into_any(),
                    Err(e) => view! { <div class="score-error">{e}</div> }.into_any(),
                })
            }}

            <button
                class="score-submit"
                class:btn--loading=move || submitting.get()
                disabled=move || submitting.get()
                on:click=on_submit
            >
                {move || {
                    if submitting.get() {
                        submitting_label.to_string()
                    } else {
                        submit_label.to_string()
                    }
                }}
            </button>
        </div>
    }
}
