use leptos::prelude::*;

#[derive(Clone)]
pub(super) struct SectionScoreState {
    pub section_id: String,
    pub section_type: String,
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
}

/// Card for scoring a single section.
#[component]
pub fn SectionScoreCard(state: SectionScoreState, focused: bool) -> impl IntoView {
    let section_type = state.section_type.clone();
    let show_notes = RwSignal::new(false);

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
                    <span class="section-score-type">{type_label}</span>
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
