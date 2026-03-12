use crate::db::Exercise;
use leptos::prelude::*;

/// A single exercise entry in the form (serialized as JSON for the server function).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct ExerciseEntry {
    exercise_id: String,
    sets: String,
    reps: String,
    weight_kg: String,
    notes: String,
}

#[server]
async fn get_exercises_for_picker() -> Result<Vec<Exercise>, ServerFnError> {
    let _user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    crate::db::list_exercises_db(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn log_workout(
    workout_date: String,
    workout_type: String,
    name: String,
    notes: String,
    duration_seconds: String,
    is_rx: bool,
    exercises_json: String,
) -> Result<(), ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let wname = if name.is_empty() {
        None
    } else {
        Some(name.as_str())
    };
    let wnotes = if notes.is_empty() {
        None
    } else {
        Some(notes.as_str())
    };
    let wduration: Option<i32> = duration_seconds.parse().ok();

    let workout_id = crate::db::create_workout_log_db(
        &pool,
        user_uuid,
        &workout_date,
        &workout_type,
        wname,
        wnotes,
        wduration,
        is_rx,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Parse exercise entries from JSON
    let entries: Vec<ExerciseEntry> = serde_json::from_str(&exercises_json).unwrap_or_default();

    for (i, entry) in entries.iter().enumerate() {
        if entry.exercise_id.is_empty() {
            continue;
        }
        let eid: uuid::Uuid = entry
            .exercise_id
            .parse()
            .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
        let sets_val: Option<i32> = entry.sets.parse().ok();
        let reps_val: Option<i32> = entry.reps.parse().ok();
        let weight_val: Option<f32> = entry.weight_kg.parse().ok();
        let enotes = if entry.notes.is_empty() {
            None
        } else {
            Some(entry.notes.as_str())
        };

        crate::db::add_workout_exercise_db(
            &pool, workout_id, eid, sets_val, reps_val, weight_val, None, i as i32, enotes,
        )
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    }

    Ok(())
}

#[component]
pub fn LogWorkoutPage() -> impl IntoView {
    let exercises_resource = Resource::new(|| (), |_| get_exercises_for_picker());
    let log_action = ServerAction::<LogWorkout>::new();

    let workout_date = RwSignal::new(String::new());
    let workout_type = RwSignal::new("strength".to_string());
    let name_input = RwSignal::new(String::new());
    let notes_input = RwSignal::new(String::new());
    let duration_input = RwSignal::new(String::new());
    let is_rx = RwSignal::new(true);
    let is_listening = RwSignal::new(false);

    // Dynamic exercise entries — start with one empty row
    let next_id = RwSignal::new(1u32);
    let exercise_rows = RwSignal::new(vec![0u32]); // list of row IDs

    // Per-row signals stored in a reactive map
    let row_exercise_id =
        StoredValue::new(std::collections::HashMap::<u32, RwSignal<String>>::new());
    let row_sets = StoredValue::new(std::collections::HashMap::<u32, RwSignal<String>>::new());
    let row_reps = StoredValue::new(std::collections::HashMap::<u32, RwSignal<String>>::new());
    let row_weight = StoredValue::new(std::collections::HashMap::<u32, RwSignal<String>>::new());
    let row_notes = StoredValue::new(std::collections::HashMap::<u32, RwSignal<String>>::new());

    // Helper to ensure signals exist for a row
    let ensure_row = move |id: u32| {
        row_exercise_id.update_value(|m| {
            m.entry(id).or_insert_with(|| RwSignal::new(String::new()));
        });
        row_sets.update_value(|m| {
            m.entry(id).or_insert_with(|| RwSignal::new(String::new()));
        });
        row_reps.update_value(|m| {
            m.entry(id).or_insert_with(|| RwSignal::new(String::new()));
        });
        row_weight.update_value(|m| {
            m.entry(id).or_insert_with(|| RwSignal::new(String::new()));
        });
        row_notes.update_value(|m| {
            m.entry(id).or_insert_with(|| RwSignal::new(String::new()));
        });
    };

    // Initialize first row
    ensure_row(0);

    let add_exercise = move |_| {
        let id = next_id.get_untracked();
        next_id.set(id + 1);
        ensure_row(id);
        exercise_rows.update(|rows| rows.push(id));
    };

    let remove_exercise = move |id: u32| {
        exercise_rows.update(|rows| rows.retain(|&r| r != id));
    };

    #[cfg(feature = "hydrate")]
    let voice_supported = RwSignal::new(false);
    #[cfg(not(feature = "hydrate"))]
    let voice_supported = RwSignal::new(false);

    #[cfg(feature = "hydrate")]
    {
        let date_signal = workout_date;
        leptos::prelude::Effect::new(move |_| {
            let today = js_sys::Date::new_0();
            let year = today.get_full_year();
            let month = today.get_month() + 1;
            let day = today.get_date();
            date_signal.set(format!("{:04}-{:02}-{:02}", year, month, day));
            voice_supported.set(crate::voice::speech_recognition_supported());
        });
    }

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        // Collect exercise entries into JSON
        let rows = exercise_rows.get_untracked();
        let entries: Vec<ExerciseEntry> = rows
            .iter()
            .filter_map(|&id| {
                let eid = row_exercise_id.with_value(|m| m.get(&id).map(|s| s.get_untracked()))?;
                let sets = row_sets
                    .with_value(|m| m.get(&id).map(|s| s.get_untracked()))
                    .unwrap_or_default();
                let reps = row_reps
                    .with_value(|m| m.get(&id).map(|s| s.get_untracked()))
                    .unwrap_or_default();
                let weight_kg = row_weight
                    .with_value(|m| m.get(&id).map(|s| s.get_untracked()))
                    .unwrap_or_default();
                let notes = row_notes
                    .with_value(|m| m.get(&id).map(|s| s.get_untracked()))
                    .unwrap_or_default();
                if eid.is_empty() {
                    return None;
                }
                Some(ExerciseEntry {
                    exercise_id: eid,
                    sets,
                    reps,
                    weight_kg,
                    notes,
                })
            })
            .collect();

        let exercises_json = serde_json::to_string(&entries).unwrap_or_default();

        log_action.dispatch(LogWorkout {
            workout_date: workout_date.get_untracked(),
            workout_type: workout_type.get_untracked(),
            name: name_input.get_untracked(),
            notes: notes_input.get_untracked(),
            duration_seconds: duration_input.get_untracked(),
            is_rx: is_rx.get_untracked(),
            exercises_json,
        });

        // Reset form
        name_input.set(String::new());
        notes_input.set(String::new());
        duration_input.set(String::new());
        // Reset exercise rows to a single empty one
        let fresh_id = next_id.get_untracked();
        next_id.set(fresh_id + 1);
        ensure_row(fresh_id);
        exercise_rows.set(vec![fresh_id]);
    };

    let log_value = log_action.value();

    view! {
        <div class="log-workout-page">
            <div class="log-header">
                <input
                    type="date"
                    class="date-picker"
                    prop:value=move || workout_date.get()
                    on:input=move |ev| workout_date.set(event_target_value(&ev))
                />
            </div>

            {move || {
                log_value.get().map(|result| {
                    match result {
                        Ok(()) => view! {
                            <div class="toast toast--success">"Result posted!"</div>
                        }.into_any(),
                        Err(e) => view! {
                            <div class="toast toast--error">{format!("Error: {}", e)}</div>
                        }.into_any(),
                    }
                })
            }}

            // Type selector pills
            <div class="type-selector">
                {["strength", "amrap", "emom", "for_time", "meditation", "breathing"].into_iter().map(|t| {
                    let t_str = t.to_string();
                    let label = match t {
                        "strength" => "Strength",
                        "amrap" => "AMRAP",
                        "emom" => "EMOM",
                        "for_time" => "For Time",
                        "meditation" => "Meditation",
                        "breathing" => "Breathing",
                        _ => t,
                    };
                    let t_active = t_str.clone();
                    let t_click = t_str.clone();
                    view! {
                        <button
                            type="button"
                            class="type-pill"
                            class:active=move || workout_type.get() == t_active
                            on:click=move |_| workout_type.set(t_click.clone())
                        >
                            {label}
                        </button>
                    }
                }).collect_view()}
            </div>

            <form class="log-form" on:submit=on_submit>
                // Workout name
                <div class="form-field">
                    <label>"Workout Name"</label>
                    <input
                        type="text"
                        placeholder="e.g. Fran, Murph, Morning Flow"
                        prop:value=move || name_input.get()
                        on:input=move |ev| name_input.set(event_target_value(&ev))
                    />
                </div>

                // Rx/Scaled toggle
                <div class="rx-toggle">
                    <button
                        type="button"
                        class="rx-btn"
                        class:active=move || is_rx.get()
                        on:click=move |_| is_rx.set(true)
                    >"Rx"</button>
                    <button
                        type="button"
                        class="rx-btn rx-btn--scaled"
                        class:active=move || !is_rx.get()
                        on:click=move |_| is_rx.set(false)
                    >"Scaled"</button>
                </div>

                // Duration
                <div class="form-field">
                    <label>"Duration (seconds)"</label>
                    <input
                        type="number"
                        placeholder="e.g. 600"
                        prop:value=move || duration_input.get()
                        on:input=move |ev| duration_input.set(event_target_value(&ev))
                    />
                </div>

                // Exercises section
                <div class="exercises-section">
                    <div class="exercises-header">
                        <h3 class="section-title">"Exercises"</h3>
                        <button type="button" class="add-exercise-btn" on:click=add_exercise>
                            <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
                            </svg>
                            "Add"
                        </button>
                    </div>

                    <Suspense fallback=|| view! { <div class="exercise-row">"Loading exercises..."</div> }>
                        {move || {
                            exercises_resource.get().map(|result| {
                                let exercise_list = result.unwrap_or_default();
                                let exercise_list = StoredValue::new(exercise_list);

                                view! {
                                    <div class="exercise-rows">
                                        <For
                                            each=move || exercise_rows.get()
                                            key=|id| *id
                                            let:row_id
                                        >
                                            {
                                                let exercises = exercise_list.get_value();
                                                let can_remove = move || exercise_rows.get().len() > 1;
                                                let rid = row_id;

                                                // Get or create signals for this row
                                                ensure_row(rid);
                                                let eid_sig = row_exercise_id.with_value(|m| m[&rid]);
                                                let sets_sig = row_sets.with_value(|m| m[&rid]);
                                                let reps_sig = row_reps.with_value(|m| m[&rid]);
                                                let weight_sig = row_weight.with_value(|m| m[&rid]);
                                                let notes_sig = row_notes.with_value(|m| m[&rid]);

                                                view! {
                                                    <div class="exercise-row">
                                                        <div class="exercise-row-header">
                                                            <select
                                                                class="exercise-select"
                                                                prop:value=move || eid_sig.get()
                                                                on:change=move |ev| eid_sig.set(event_target_value(&ev))
                                                            >
                                                                <option value="">"Select exercise"</option>
                                                                {exercises.iter().map(|ex| {
                                                                    let id = ex.id.clone();
                                                                    let name = ex.name.clone();
                                                                    view! { <option value={id}>{name}</option> }
                                                                }).collect_view()}
                                                            </select>
                                                            <Show when=can_remove>
                                                                <button
                                                                    type="button"
                                                                    class="remove-exercise-btn"
                                                                    on:click=move |_| remove_exercise(rid)
                                                                >
                                                                    <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                                                                        <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
                                                                    </svg>
                                                                </button>
                                                            </Show>
                                                        </div>
                                                        <div class="metrics-row">
                                                            <div class="metric">
                                                                <label>"Sets"</label>
                                                                <input
                                                                    type="number"
                                                                    placeholder="--"
                                                                    prop:value=move || sets_sig.get()
                                                                    on:input=move |ev| sets_sig.set(event_target_value(&ev))
                                                                />
                                                            </div>
                                                            <div class="metric">
                                                                <label>"Reps"</label>
                                                                <input
                                                                    type="number"
                                                                    placeholder="--"
                                                                    prop:value=move || reps_sig.get()
                                                                    on:input=move |ev| reps_sig.set(event_target_value(&ev))
                                                                />
                                                            </div>
                                                            <div class="metric">
                                                                <label>"Weight"</label>
                                                                <div class="weight-wrap">
                                                                    <input
                                                                        type="number"
                                                                        step="0.5"
                                                                        placeholder="--"
                                                                        prop:value=move || weight_sig.get()
                                                                        on:input=move |ev| weight_sig.set(event_target_value(&ev))
                                                                    />
                                                                    <span class="weight-unit">"kg"</span>
                                                                </div>
                                                            </div>
                                                        </div>
                                                        <input
                                                            type="text"
                                                            class="exercise-notes"
                                                            placeholder="Exercise notes (optional)"
                                                            prop:value=move || notes_sig.get()
                                                            on:input=move |ev| notes_sig.set(event_target_value(&ev))
                                                        />
                                                    </div>
                                                }
                                            }
                                        </For>
                                    </div>
                                }
                            })
                        }}
                    </Suspense>
                </div>

                // Notes
                <div class="form-field">
                    <label>"Notes"</label>
                    <textarea
                        placeholder="How did it feel? Any PRs?"
                        prop:value=move || notes_input.get()
                        on:input=move |ev| notes_input.set(event_target_value(&ev))
                    />
                </div>

                <button type="submit" class="submit-btn">"Post Result"</button>
            </form>

            // Sticky voice FAB
            {move || voice_supported.get().then(|| view! {
                <button
                    type="button"
                    class="mic-fab"
                    class:listening=move || is_listening.get()
                    on:click=move |_| {
                        #[cfg(feature = "hydrate")]
                        {
                            if is_listening.get_untracked() {
                                return;
                            }
                            is_listening.set(true);

                            let on_result = wasm_bindgen::closure::Closure::wrap(Box::new(move |text: String| {
                                let current = notes_input.get_untracked();
                                if current.is_empty() {
                                    notes_input.set(text);
                                } else {
                                    notes_input.set(format!("{} {}", current, text));
                                }
                            }) as Box<dyn Fn(String)>);

                            let on_error = wasm_bindgen::closure::Closure::wrap(Box::new(move |err: String| {
                                leptos::logging::log!("Speech error: {}", err);
                            }) as Box<dyn Fn(String)>);

                            let on_end = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                                is_listening.set(false);
                            }) as Box<dyn Fn()>);

                            let _ = crate::voice::start_speech(&on_result, &on_error, &on_end);

                            on_result.forget();
                            on_error.forget();
                            on_end.forget();
                        }
                    }
                >
                    <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                        <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3z"/>
                        <path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z"/>
                    </svg>
                    <span class="mic-fab-label">{move || if is_listening.get() { "Listening..." } else { "Log" }}</span>
                </button>
            })}
        </div>
    }
}
