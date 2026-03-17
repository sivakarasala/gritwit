use crate::auth::AuthUser;
use crate::db::MovementLog;
use leptos::prelude::*;

/// Per-set row state (for multi-set rep schemes like "9-8-7-6" or "5x5").
#[derive(Clone)]
pub(super) struct MovementSetState {
    pub set_number: i32,
    pub reps: RwSignal<String>,
    pub weight_kg: RwSignal<String>,
    pub distance_meters: RwSignal<String>,
    pub calories: RwSignal<String>,
}

/// Reactive state for a single movement's log inputs.
#[derive(Clone)]
pub(super) struct MovementLogState {
    pub movement_id: String,
    pub exercise_name: String,
    pub scoring_type: String,
    pub prescribed_reps: Option<String>,
    pub prescribed_weight_male: Option<f32>,
    pub prescribed_weight_female: Option<f32>,
    pub reps: RwSignal<String>,
    pub sets: RwSignal<String>,
    pub weight_kg: RwSignal<String>,
    pub distance_meters: RwSignal<String>,
    pub calories: RwSignal<String>,
    pub duration_seconds: RwSignal<String>,
    pub notes: RwSignal<String>,
    /// Per-set rows. When non-empty, UI renders these instead of the flat reps/sets/weight inputs.
    pub set_rows: Vec<MovementSetState>,
}

#[derive(Clone)]
pub(super) struct SectionScoreState {
    pub section_id: String,
    pub section_type: String,
    pub phase: String,
    pub title: String,
    pub time_cap: Option<i32>,
    #[allow(dead_code)]
    pub rounds: Option<i32>,
    pub is_rx: RwSignal<bool>,
    pub skipped: RwSignal<bool>,
    pub minutes: RwSignal<String>,
    pub seconds: RwSignal<String>,
    pub rounds_completed: RwSignal<String>,
    pub extra_reps: RwSignal<String>,
    pub weight_kg: RwSignal<String>,
    pub notes: RwSignal<String>,
    pub movement_states: RwSignal<Vec<MovementLogState>>,
    /// Pre-saved movement logs from a previous score (for edit mode).
    pub existing_movement_logs: Vec<MovementLog>,
}

/// Card for scoring a single section.
#[component]
pub fn SectionScoreCard(state: SectionScoreState, focused: bool) -> impl IntoView {
    let section_type = state.section_type.clone();
    let is_conditioning = state.phase == "conditioning";
    let show_notes = RwSignal::new(false);

    // Load movements for this section independently
    let sec_id = state.section_id.clone();
    let movement_states_signal = state.movement_states;
    let existing_mov_logs = state.existing_movement_logs.clone();
    let movements = Resource::new(
        move || sec_id.clone(),
        super::server_fns::get_section_movements_for_log,
    );

    // Get user gender from context for weight pre-population
    let auth_user = use_context::<AuthUser>();
    let user_gender = auth_user.as_ref().and_then(|u| u.gender.clone());
    let gender_not_set = auth_user
        .as_ref()
        .map(|u| u.gender.is_none())
        .unwrap_or(true);
    let show_gender_hint = RwSignal::new(false);

    // When movements load, initialize movement states with pre-populated values.
    Effect::new(move || {
        if let Some(Ok(movs)) = movements.get() {
            if movement_states_signal.get_untracked().is_empty() && !movs.is_empty() {
                let gender = user_gender.clone();
                let existing = &existing_mov_logs;
                let has_gendered_weights = gender_not_set
                    && movs.iter().any(|m| {
                        m.weight_kg_male.is_some()
                            && m.weight_kg_female.is_some()
                            && m.weight_kg_male != m.weight_kg_female
                    });
                if has_gendered_weights {
                    show_gender_hint.set(true);
                }

                let states: Vec<MovementLogState> = movs
                    .into_iter()
                    .map(|m| {
                        let saved = existing.iter().find(|ml| ml.movement_id == m.id);
                        let pre_weight = match gender.as_deref() {
                            Some("female") => m.weight_kg_female.or(m.weight_kg_male),
                            _ => m.weight_kg_male.or(m.weight_kg_female),
                        };
                        let pre_weight_str = pre_weight.map(|w| w.to_string()).unwrap_or_default();

                        let per_set_reps = parse_rep_scheme(m.rep_scheme.as_deref());
                        let per_set_distances = parse_distance_scheme(m.rep_scheme.as_deref());
                        let per_set_calories = parse_calories_scheme(m.rep_scheme.as_deref());
                        // Per-set rows for weight_and_reps and reps_only with 2+ sets
                        let set_rows = if (m.scoring_type == "weight_and_reps"
                            || m.scoring_type == "reps_only")
                            && per_set_reps.len() >= 2
                        {
                            per_set_reps
                                .iter()
                                .enumerate()
                                .map(|(i, &reps)| MovementSetState {
                                    set_number: (i + 1) as i32,
                                    reps: RwSignal::new(reps.to_string()),
                                    weight_kg: RwSignal::new(if let Some(log) = saved {
                                        log.weight_kg
                                            .map(|w| w.to_string())
                                            .unwrap_or_else(|| pre_weight_str.clone())
                                    } else {
                                        pre_weight_str.clone()
                                    }),
                                    distance_meters: RwSignal::new(String::new()),
                                    calories: RwSignal::new(String::new()),
                                })
                                .collect()
                        } else if m.scoring_type == "distance" && per_set_distances.len() >= 2 {
                            per_set_distances
                                .iter()
                                .enumerate()
                                .map(|(i, &dist)| MovementSetState {
                                    set_number: (i + 1) as i32,
                                    reps: RwSignal::new(String::new()),
                                    weight_kg: RwSignal::new(String::new()),
                                    distance_meters: RwSignal::new(dist.to_string()),
                                    calories: RwSignal::new(String::new()),
                                })
                                .collect()
                        } else if m.scoring_type == "calories" && per_set_calories.len() >= 2 {
                            per_set_calories
                                .iter()
                                .enumerate()
                                .map(|(i, &cal)| MovementSetState {
                                    set_number: (i + 1) as i32,
                                    reps: RwSignal::new(String::new()),
                                    weight_kg: RwSignal::new(String::new()),
                                    distance_meters: RwSignal::new(String::new()),
                                    calories: RwSignal::new(cal.to_string()),
                                })
                                .collect()
                        } else {
                            Vec::new()
                        };

                        if let Some(log) = saved {
                            MovementLogState {
                                movement_id: m.id.clone(),
                                exercise_name: m.exercise_name.clone(),
                                scoring_type: m.scoring_type.clone(),
                                prescribed_reps: m.rep_scheme.clone(),
                                prescribed_weight_male: m.weight_kg_male,
                                prescribed_weight_female: m.weight_kg_female,
                                reps: RwSignal::new(
                                    log.reps.map(|r| r.to_string()).unwrap_or_default(),
                                ),
                                sets: RwSignal::new(
                                    log.sets.map(|s| s.to_string()).unwrap_or_default(),
                                ),
                                weight_kg: RwSignal::new(
                                    log.weight_kg.map(|w| w.to_string()).unwrap_or_default(),
                                ),
                                distance_meters: RwSignal::new(String::new()),
                                calories: RwSignal::new(String::new()),
                                duration_seconds: RwSignal::new(String::new()),
                                notes: RwSignal::new(log.notes.clone().unwrap_or_default()),
                                set_rows,
                            }
                        } else {
                            let single_reps = per_set_reps.first().copied();
                            let single_sets = if per_set_reps.len() >= 2 {
                                Some(per_set_reps.len() as i32)
                            } else {
                                None
                            };
                            MovementLogState {
                                movement_id: m.id.clone(),
                                exercise_name: m.exercise_name.clone(),
                                scoring_type: m.scoring_type.clone(),
                                prescribed_reps: m.rep_scheme.clone(),
                                prescribed_weight_male: m.weight_kg_male,
                                prescribed_weight_female: m.weight_kg_female,
                                reps: RwSignal::new(
                                    single_reps.map(|r| r.to_string()).unwrap_or_default(),
                                ),
                                sets: RwSignal::new(
                                    single_sets.map(|s| s.to_string()).unwrap_or_default(),
                                ),
                                weight_kg: RwSignal::new(pre_weight_str),
                                distance_meters: RwSignal::new(String::new()),
                                calories: RwSignal::new(String::new()),
                                duration_seconds: RwSignal::new(String::new()),
                                notes: RwSignal::new(String::new()),
                                set_rows,
                            }
                        }
                    })
                    .collect();
                movement_states_signal.set(states);
            }
        }
    });

    let type_label = match section_type.as_str() {
        "fortime" => "For Time",
        "amrap" => "AMRAP",
        "emom" => "EMOM",
        "strength" => "Strength",
        _ => "Other",
    };

    let cap_info = match section_type.as_str() {
        "fortime" => state
            .time_cap
            .map(|c| format!("Time cap: {} min", c))
            .unwrap_or_default(),
        "amrap" | "emom" => state
            .time_cap
            .map(|c| format!("{} min", c))
            .unwrap_or_default(),
        _ => String::new(),
    };

    view! {
        <div class="section-score-card" class:focused=focused>
            <div class="section-score-header">
                <div class="section-score-info">
                    <span class="section-score-title">{state.title.clone()}</span>
                    {is_conditioning.then(|| view! {
                        <span class="section-score-type">{type_label}</span>
                    })}
                    {(!cap_info.is_empty()).then(|| view! {
                        <span class="section-score-cap">{cap_info.clone()}</span>
                    })}
                </div>
                <div class="section-score-toggles">
                    <button
                        class="rx-toggle"
                        class:rx-on=move || state.is_rx.get()
                        on:click=move |_| state.is_rx.update(|v| *v = !*v)
                        disabled=move || state.skipped.get()
                    >
                        {move || if state.is_rx.get() { "RX" } else { "Scaled" }}
                    </button>
                    <button
                        class="skip-toggle"
                        class:skip-on=move || state.skipped.get()
                        on:click=move |_| state.skipped.update(|v| *v = !*v)
                    >"Skip"</button>
                </div>
            </div>

            // Movement inputs
            <div class="section-movements-log" class:skipped=move || state.skipped.get()>
                {move || {
                    let ms = movement_states_signal.get();
                    if ms.is_empty() {
                        None
                    } else {
                        Some(view! {
                            <div class="section-mov-cards">
                                {ms.into_iter().map(|m| {
                                    let rx_weight = if m.scoring_type == "weight_and_reps" {
                                        format_prescribed_weight(
                                            m.prescribed_weight_male,
                                            m.prescribed_weight_female,
                                        )
                                    } else {
                                        String::new()
                                    };
                                    let has_set_rows = !m.set_rows.is_empty();
                                    let scoring_type = m.scoring_type.clone();
                                    view! {
                                        <div class="mov-log-card">
                                            <div class="mov-log-header">
                                                <span class="mov-log-name">{m.exercise_name.clone()}</span>
                                                {m.prescribed_reps.clone().map(|r| view! {
                                                    <span class="mov-log-prescribed">{r}</span>
                                                })}
                                                {(!rx_weight.is_empty()).then(|| view! {
                                                    <span class="mov-log-rx-weight">{rx_weight.clone()}</span>
                                                })}
                                            </div>
                                            {match scoring_type.as_str() {
                                                "distance" => if has_set_rows {
                                                    view! {
                                                        <div class="mov-set-rows">
                                                            {m.set_rows.into_iter().map(|sr| {
                                                                let label = format!("Set {}", sr.set_number);
                                                                view! {
                                                                    <div class="mov-set-row">
                                                                        <span class="mov-set-label">{label}</span>
                                                                        <div class="mov-log-inputs">
                                                                            <div class="mov-log-field">
                                                                                <input
                                                                                    type="number"
                                                                                    class="mov-input"
                                                                                    placeholder="Distance"
                                                                                    inputmode="decimal"
                                                                                    step="1"
                                                                                    min="0"
                                                                                    prop:value=move || sr.distance_meters.get()
                                                                                    on:input=move |ev| sr.distance_meters.set(event_target_value(&ev))
                                                                                />
                                                                                <span class="mov-input-label">"m"</span>
                                                                            </div>
                                                                        </div>
                                                                    </div>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="mov-log-inputs">
                                                            <div class="mov-log-field">
                                                                <input
                                                                    type="number"
                                                                    class="mov-input"
                                                                    placeholder="Distance"
                                                                    inputmode="decimal"
                                                                    step="1"
                                                                    min="0"
                                                                    prop:value=move || m.distance_meters.get()
                                                                    on:input=move |ev| m.distance_meters.set(event_target_value(&ev))
                                                                />
                                                                <span class="mov-input-label">"m"</span>
                                                            </div>
                                                        </div>
                                                    }.into_any()
                                                },
                                                "calories" => if has_set_rows {
                                                    view! {
                                                        <div class="mov-set-rows">
                                                            {m.set_rows.into_iter().map(|sr| {
                                                                let label = format!("Set {}", sr.set_number);
                                                                view! {
                                                                    <div class="mov-set-row">
                                                                        <span class="mov-set-label">{label}</span>
                                                                        <div class="mov-log-inputs">
                                                                            <div class="mov-log-field">
                                                                                <input
                                                                                    type="number"
                                                                                    class="mov-input"
                                                                                    placeholder="Calories"
                                                                                    inputmode="numeric"
                                                                                    step="1"
                                                                                    min="0"
                                                                                    prop:value=move || sr.calories.get()
                                                                                    on:input=move |ev| sr.calories.set(event_target_value(&ev))
                                                                                />
                                                                                <span class="mov-input-label">"cal"</span>
                                                                            </div>
                                                                        </div>
                                                                    </div>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="mov-log-inputs">
                                                            <div class="mov-log-field">
                                                                <input
                                                                    type="number"
                                                                    class="mov-input"
                                                                    placeholder="Calories"
                                                                    inputmode="numeric"
                                                                    step="1"
                                                                    min="0"
                                                                    prop:value=move || m.calories.get()
                                                                    on:input=move |ev| m.calories.set(event_target_value(&ev))
                                                                />
                                                                <span class="mov-input-label">"cal"</span>
                                                            </div>
                                                        </div>
                                                    }.into_any()
                                                },
                                                "time" => view! {
                                                    <div class="mov-log-inputs">
                                                        <div class="mov-log-field">
                                                            <input
                                                                type="number"
                                                                class="mov-input"
                                                                placeholder="Seconds"
                                                                inputmode="numeric"
                                                                step="1"
                                                                min="0"
                                                                prop:value=move || m.duration_seconds.get()
                                                                on:input=move |ev| m.duration_seconds.set(event_target_value(&ev))
                                                            />
                                                            <span class="mov-input-label">"s"</span>
                                                        </div>
                                                    </div>
                                                }.into_any(),
                                                "reps_only" => if has_set_rows {
                                                    view! {
                                                        <div class="mov-set-rows">
                                                            {m.set_rows.into_iter().map(|sr| {
                                                                let label = format!("Set {}", sr.set_number);
                                                                view! {
                                                                    <div class="mov-set-row">
                                                                        <span class="mov-set-label">{label}</span>
                                                                        <div class="mov-log-inputs">
                                                                            <div class="mov-log-field">
                                                                                <input
                                                                                    type="number"
                                                                                    class="mov-input"
                                                                                    placeholder="Reps"
                                                                                    inputmode="numeric"
                                                                                    min="0"
                                                                                    prop:value=move || sr.reps.get()
                                                                                    on:input=move |ev| sr.reps.set(event_target_value(&ev))
                                                                                />
                                                                                <span class="mov-input-label">"reps"</span>
                                                                            </div>
                                                                        </div>
                                                                    </div>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="mov-log-inputs">
                                                            <div class="mov-log-field">
                                                                <input
                                                                    type="number"
                                                                    class="mov-input"
                                                                    placeholder="Reps"
                                                                    inputmode="numeric"
                                                                    min="0"
                                                                    prop:value=move || m.reps.get()
                                                                    on:input=move |ev| m.reps.set(event_target_value(&ev))
                                                                />
                                                                <span class="mov-input-label">"reps"</span>
                                                            </div>
                                                            <div class="mov-log-field">
                                                                <input
                                                                    type="number"
                                                                    class="mov-input"
                                                                    placeholder="Sets"
                                                                    inputmode="numeric"
                                                                    min="0"
                                                                    prop:value=move || m.sets.get()
                                                                    on:input=move |ev| m.sets.set(event_target_value(&ev))
                                                                />
                                                                <span class="mov-input-label">"sets"</span>
                                                            </div>
                                                        </div>
                                                    }.into_any()
                                                },
                                                // weight_and_reps (default)
                                                _ => if has_set_rows {
                                                    view! {
                                                        <div class="mov-set-rows">
                                                            {m.set_rows.into_iter().map(|sr| {
                                                                let label = format!("Set {}", sr.set_number);
                                                                view! {
                                                                    <div class="mov-set-row">
                                                                        <span class="mov-set-label">{label}</span>
                                                                        <div class="mov-log-inputs">
                                                                            <div class="mov-log-field">
                                                                                <input
                                                                                    type="number"
                                                                                    class="mov-input"
                                                                                    placeholder="Reps"
                                                                                    inputmode="numeric"
                                                                                    min="0"
                                                                                    prop:value=move || sr.reps.get()
                                                                                    on:input=move |ev| sr.reps.set(event_target_value(&ev))
                                                                                />
                                                                                <span class="mov-input-label">"reps"</span>
                                                                            </div>
                                                                            <div class="mov-log-field">
                                                                                <input
                                                                                    type="number"
                                                                                    class="mov-input"
                                                                                    placeholder="kg"
                                                                                    inputmode="decimal"
                                                                                    step="0.5"
                                                                                    min="0"
                                                                                    prop:value=move || sr.weight_kg.get()
                                                                                    on:input=move |ev| sr.weight_kg.set(event_target_value(&ev))
                                                                                />
                                                                                <span class="mov-input-label">"kg"</span>
                                                                            </div>
                                                                        </div>
                                                                    </div>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="mov-log-inputs">
                                                            <div class="mov-log-field">
                                                                <input
                                                                    type="number"
                                                                    class="mov-input"
                                                                    placeholder="Reps"
                                                                    inputmode="numeric"
                                                                    min="0"
                                                                    prop:value=move || m.reps.get()
                                                                    on:input=move |ev| m.reps.set(event_target_value(&ev))
                                                                />
                                                                <span class="mov-input-label">"reps"</span>
                                                            </div>
                                                            <div class="mov-log-field">
                                                                <input
                                                                    type="number"
                                                                    class="mov-input"
                                                                    placeholder="Sets"
                                                                    inputmode="numeric"
                                                                    min="0"
                                                                    prop:value=move || m.sets.get()
                                                                    on:input=move |ev| m.sets.set(event_target_value(&ev))
                                                                />
                                                                <span class="mov-input-label">"sets"</span>
                                                            </div>
                                                            <div class="mov-log-field">
                                                                <input
                                                                    type="number"
                                                                    class="mov-input"
                                                                    placeholder="kg"
                                                                    inputmode="decimal"
                                                                    step="0.5"
                                                                    min="0"
                                                                    prop:value=move || m.weight_kg.get()
                                                                    on:input=move |ev| m.weight_kg.set(event_target_value(&ev))
                                                                />
                                                                <span class="mov-input-label">"kg"</span>
                                                            </div>
                                                        </div>
                                                    }.into_any()
                                                },
                                            }}
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        })
                    }
                }}
            </div>

            {move || show_gender_hint.get().then(|| view! {
                <div class="gender-hint">
                    "Weights default to male Rx. "
                    <a href="/profile" class="gender-hint-link">"Set your gender in Profile"</a>
                    " for personalized weights."
                </div>
            })}

            <div class="section-score-body" class:skipped=move || state.skipped.get()>
                {match section_type.as_str() {
                    "fortime" => view! {
                        <div class="time-inputs">
                            <div class="time-field">
                                <input
                                    type="number"
                                    class="score-input"
                                    placeholder="0"
                                    inputmode="numeric"
                                    min="0"
                                    prop:value=move || state.minutes.get()
                                    on:input=move |ev| state.minutes.set(event_target_value(&ev))
                                />
                                <span class="time-unit">"min"</span>
                            </div>
                            <span class="time-sep">":"</span>
                            <div class="time-field">
                                <input
                                    type="number"
                                    class="score-input"
                                    placeholder="00"
                                    inputmode="numeric"
                                    min="0"
                                    max="59"
                                    prop:value=move || state.seconds.get()
                                    on:input=move |ev| state.seconds.set(event_target_value(&ev))
                                />
                                <span class="time-unit">"sec"</span>
                            </div>
                        </div>
                    }.into_any(),
                    "amrap" | "emom" => view! {
                        <div class="rounds-inputs">
                            <div class="rounds-field">
                                <input
                                    type="number"
                                    class="score-input"
                                    placeholder="0"
                                    inputmode="numeric"
                                    min="0"
                                    prop:value=move || state.rounds_completed.get()
                                    on:input=move |ev| state.rounds_completed.set(event_target_value(&ev))
                                />
                                <span class="rounds-unit">"rounds"</span>
                            </div>
                            <span class="rounds-sep">"+"</span>
                            <div class="rounds-field">
                                <input
                                    type="number"
                                    class="score-input"
                                    placeholder="0"
                                    inputmode="numeric"
                                    min="0"
                                    prop:value=move || state.extra_reps.get()
                                    on:input=move |ev| state.extra_reps.set(event_target_value(&ev))
                                />
                                <span class="rounds-unit">"reps"</span>
                            </div>
                        </div>
                    }.into_any(),
                    "strength" => view! {
                        <div class="weight-input">
                            <p class="score-helper-text">"Top weight for this session (e.g. your heavy single or working max)"</p>
                            <div class="weight-field">
                                <input
                                    type="number"
                                    class="score-input"
                                    placeholder="0"
                                    inputmode="decimal"
                                    step="0.5"
                                    min="0"
                                    prop:value=move || state.weight_kg.get()
                                    on:input=move |ev| state.weight_kg.set(event_target_value(&ev))
                                />
                                <span class="weight-unit">"kg"</span>
                            </div>
                        </div>
                    }.into_any(),
                    _ => view! {
                        <p class="section-score-static">"No score needed"</p>
                    }.into_any(),
                }}

                <div class="section-score-extras">
                    <button
                        class="notes-toggle"
                        on:click=move |_| show_notes.update(|v| *v = !*v)
                    >
                        {move || if show_notes.get() { "Hide notes" } else { "Add notes" }}
                    </button>
                    {move || show_notes.get().then(|| view! {
                        <textarea
                            class="section-notes"
                            placeholder="Section notes..."
                            prop:value=move || state.notes.get()
                            on:input=move |ev| state.notes.set(event_target_value(&ev))
                        ></textarea>
                    })}
                </div>
            </div>
        </div>
    }
}

fn format_prescribed_weight(male: Option<f32>, female: Option<f32>) -> String {
    match (male, female) {
        (Some(m), Some(f)) => format!("Rx: {}kg / {}kg", m, f),
        (Some(m), None) => format!("Rx: {}kg", m),
        (None, Some(f)) => format!("Rx: {}kg", f),
        (None, None) => String::new(),
    }
}

/// Parse a rep scheme string into per-set rep counts.
fn parse_rep_scheme(scheme: Option<&str>) -> Vec<i32> {
    let Some(s) = scheme else {
        return Vec::new();
    };
    let s = s.trim();

    if let Some((left, right)) = s.split_once('x').or_else(|| s.split_once('X')) {
        if let (Ok(sets), Ok(reps)) = (left.trim().parse::<i32>(), right.trim().parse::<i32>()) {
            return vec![reps; sets as usize];
        }
    }

    if s.contains('-') {
        let parts: Vec<i32> = s.split('-').filter_map(|p| p.trim().parse().ok()).collect();
        if parts.len() >= 2 {
            return parts;
        }
    }

    if let Ok(n) = s.parse::<i32>() {
        return vec![n];
    }

    Vec::new()
}

/// Parse a calories scheme like "12-10-8" or "4x12cal" into per-set calorie targets.
fn parse_calories_scheme(scheme: Option<&str>) -> Vec<i32> {
    let Some(s) = scheme else {
        return Vec::new();
    };
    let s = s.trim();

    let parse_cal = |p: &str| -> Option<i32> {
        let stripped = p.trim().trim_end_matches(|c: char| c.is_alphabetic());
        stripped.trim().parse::<i32>().ok()
    };

    if let Some((left, right)) = s.split_once('x').or_else(|| s.split_once('X')) {
        if let (Ok(sets), Some(cal)) = (left.trim().parse::<i32>(), parse_cal(right)) {
            return vec![cal; sets as usize];
        }
    }

    if s.contains('-') {
        let parts: Vec<i32> = s.split('-').filter_map(parse_cal).collect();
        if parts.len() >= 2 {
            return parts;
        }
    }

    Vec::new()
}

/// Parse a distance scheme like "500m-500m-400m" or "4x500m" into per-set distances.
fn parse_distance_scheme(scheme: Option<&str>) -> Vec<f32> {
    let Some(s) = scheme else {
        return Vec::new();
    };
    let s = s.trim();

    // Strip trailing unit (m, km, etc.) helper
    let parse_dist = |p: &str| -> Option<f32> {
        let stripped = p.trim().trim_end_matches(|c: char| c.is_alphabetic());
        stripped.trim().parse::<f32>().ok()
    };

    // "4x500m" format
    if let Some((left, right)) = s.split_once('x').or_else(|| s.split_once('X')) {
        if let (Ok(sets), Some(dist)) = (left.trim().parse::<i32>(), parse_dist(right)) {
            return vec![dist; sets as usize];
        }
    }

    // "500m-400m-300m" format
    if s.contains('-') {
        let parts: Vec<f32> = s.split('-').filter_map(parse_dist).collect();
        if parts.len() >= 2 {
            return parts;
        }
    }

    Vec::new()
}
