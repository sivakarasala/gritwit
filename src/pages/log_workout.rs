use crate::db::Exercise;
use leptos::prelude::*;

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
    exercise_id: String,
    sets: String,
    reps: String,
    weight_kg: String,
    is_rx: bool,
) -> Result<(), ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user.id.parse().map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let wname = if name.is_empty() { None } else { Some(name.as_str()) };
    let wnotes = if notes.is_empty() { None } else { Some(notes.as_str()) };
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

    if !exercise_id.is_empty() {
        let eid: uuid::Uuid = exercise_id
            .parse()
            .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
        let sets_val: Option<i32> = sets.parse().ok();
        let reps_val: Option<i32> = reps.parse().ok();
        let weight_val: Option<f32> = weight_kg.parse().ok();

        crate::db::add_workout_exercise_db(
            &pool,
            workout_id,
            eid,
            sets_val,
            reps_val,
            weight_val,
            None,
            0,
            None,
        )
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    }

    Ok(())
}

#[component]
pub fn LogWorkoutPage() -> impl IntoView {
    let exercises = Resource::new(|| (), |_| get_exercises_for_picker());
    let log_action = ServerAction::<LogWorkout>::new();

    let workout_date = RwSignal::new(String::new());
    let workout_type = RwSignal::new("strength".to_string());
    let name_input = RwSignal::new(String::new());
    let notes_input = RwSignal::new(String::new());
    let duration_input = RwSignal::new(String::new());
    let exercise_id = RwSignal::new(String::new());
    let sets_input = RwSignal::new(String::new());
    let reps_input = RwSignal::new(String::new());
    let weight_input = RwSignal::new(String::new());
    let is_rx = RwSignal::new(true);

    #[cfg(feature = "hydrate")]
    {
        let date_signal = workout_date;
        leptos::prelude::Effect::new(move |_| {
            let today = js_sys::Date::new_0();
            let year = today.get_full_year();
            let month = today.get_month() + 1;
            let day = today.get_date();
            date_signal.set(format!("{:04}-{:02}-{:02}", year, month, day));
        });
    }

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        log_action.dispatch(LogWorkout {
            workout_date: workout_date.get_untracked(),
            workout_type: workout_type.get_untracked(),
            name: name_input.get_untracked(),
            notes: notes_input.get_untracked(),
            duration_seconds: duration_input.get_untracked(),
            exercise_id: exercise_id.get_untracked(),
            sets: sets_input.get_untracked(),
            reps: reps_input.get_untracked(),
            weight_kg: weight_input.get_untracked(),
            is_rx: is_rx.get_untracked(),
        });
        name_input.set(String::new());
        notes_input.set(String::new());
        duration_input.set(String::new());
        sets_input.set(String::new());
        reps_input.set(String::new());
        weight_input.set(String::new());
    };

    let log_value = log_action.value();

    view! {
        <div class="log-workout-page">
            <div class="log-header">
                <h1>"Log Result"</h1>
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
                {["strength", "amrap", "emom", "for_time", "meditation", "breathing", "chanting"].into_iter().map(|t| {
                    let t_str = t.to_string();
                    let label = match t {
                        "strength" => "Strength",
                        "amrap" => "AMRAP",
                        "emom" => "EMOM",
                        "for_time" => "For Time",
                        "meditation" => "Meditation",
                        "breathing" => "Breathing",
                        "chanting" => "Chanting",
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

                // Movement details section
                <div class="movement-section">
                    <h3 class="section-title">"Movement Details"</h3>
                    <div class="form-field">
                        <Suspense fallback=|| view! { <select><option>"Loading..."</option></select> }>
                            {move || {
                                exercises.get().map(|result| {
                                    match result {
                                        Ok(list) => view! {
                                            <select
                                                class="exercise-select"
                                                prop:value=move || exercise_id.get()
                                                on:change=move |ev| exercise_id.set(event_target_value(&ev))
                                            >
                                                <option value="">"Select movement"</option>
                                                {list.into_iter().map(|ex| {
                                                    view! { <option value={ex.id.clone()}>{ex.name}</option> }
                                                }).collect_view()}
                                            </select>
                                        }.into_any(),
                                        Err(_) => view! { <select><option>"Error loading"</option></select> }.into_any(),
                                    }
                                })
                            }}
                        </Suspense>
                    </div>
                    <div class="metrics-row">
                        <div class="metric">
                            <label>"Sets"</label>
                            <input
                                type="number"
                                placeholder="--"
                                prop:value=move || sets_input.get()
                                on:input=move |ev| sets_input.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="metric">
                            <label>"Reps"</label>
                            <input
                                type="number"
                                placeholder="--"
                                prop:value=move || reps_input.get()
                                on:input=move |ev| reps_input.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="metric">
                            <label>"Weight"</label>
                            <div class="weight-wrap">
                                <input
                                    type="number"
                                    step="0.5"
                                    placeholder="--"
                                    prop:value=move || weight_input.get()
                                    on:input=move |ev| weight_input.set(event_target_value(&ev))
                                />
                                <span class="weight-unit">"kg"</span>
                            </div>
                        </div>
                    </div>
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
        </div>
    }
}
