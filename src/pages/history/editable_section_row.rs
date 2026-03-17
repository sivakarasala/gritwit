use crate::db::SectionScoreWithMeta;
use leptos::prelude::*;

use super::update_section_score;

#[component]
pub(super) fn EditableSectionRow(s: SectionScoreWithMeta) -> impl IntoView {
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
                "fortime" | "static" => {
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
                    "fortime" | "static" => view! {
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

fn format_section_type(t: &str) -> String {
    match t {
        "fortime" => "For Time".to_string(),
        "amrap" => "AMRAP".to_string(),
        "emom" => "EMOM".to_string(),
        "strength" => "Strength".to_string(),
        "static" => "Static".to_string(),
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
        "fortime" | "static" => {
            if let Some(t) = finish_time {
                format!("{}:{:02}", t / 60, t % 60)
            } else {
                "No score".to_string()
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
                "No score".to_string()
            }
        }
        _ => "No score".to_string(),
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
