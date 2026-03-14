pub(super) mod custom_log;
mod exercise_entry_card;
mod section_score_card;
mod server_fns;
mod set_row;
mod wod_score_form;

use custom_log::CustomLogFlow;
use leptos::prelude::*;
use server_fns::*;
use wod_score_form::WodScoreForm;

// ---- Page Component ----

#[component]
pub fn LogWorkoutPage() -> impl IntoView {
    let params = leptos_router::hooks::use_query_map();

    let section_id = Memo::new(move |_| {
        params
            .read()
            .get("section_id")
            .unwrap_or_default()
            .to_string()
    });
    let wod_id_param =
        Memo::new(move |_| params.read().get("wod_id").unwrap_or_default().to_string());
    let edit_id = Memo::new(move |_| params.read().get("edit").unwrap_or_default().to_string());
    let edit_log_id = Memo::new(move |_| {
        params
            .read()
            .get("edit_log")
            .unwrap_or_default()
            .to_string()
    });

    let initial_tab = if !params
        .read_untracked()
        .get("edit")
        .unwrap_or_default()
        .is_empty()
    {
        "custom"
    } else {
        "wod"
    };
    let active_tab = RwSignal::new(initial_tab.to_string());

    view! {
        <div class="log-workout-page">
            <div class="log-tabs">
                <button
                    class="log-tab"
                    class:active=move || active_tab.get() == "wod"
                    on:click=move |_| active_tab.set("wod".to_string())
                >"WOD Score"</button>
                <button
                    class="log-tab"
                    class:active=move || active_tab.get() == "custom"
                    on:click=move |_| active_tab.set("custom".to_string())
                >"Custom Log"</button>
            </div>

            {move || {
                if active_tab.get() == "wod" {
                    view! { <WodScoreFlow section_id=section_id.get() wod_id=wod_id_param.get() edit_log_id=edit_log_id.get()/> }.into_any()
                } else {
                    view! { <CustomLogFlow edit_id=edit_id.get()/> }.into_any()
                }
            }}
        </div>
    }
}

/// WOD scoring flow: either loads via section_id or wod_id, or shows a WOD picker.
#[component]
fn WodScoreFlow(section_id: String, wod_id: String, edit_log_id: String) -> impl IntoView {
    let selected_wod_id = RwSignal::new(wod_id.clone());
    let focus_section_id = section_id.clone();
    let edit_log_signal = RwSignal::new(edit_log_id.clone());

    let resolved_wod = Resource::new(
        move || (section_id.clone(), selected_wod_id.get()),
        |(sid, wid)| async move {
            if !sid.is_empty() {
                return get_wod_by_section(sid).await.map(Some);
            }
            if !wid.is_empty() {
                return get_wod_for_scoring(wid).await.map(Some);
            }
            Ok(None)
        },
    );

    let existing_scores = Resource::new(
        move || edit_log_id.clone(),
        |lid| async move {
            if lid.is_empty() {
                return Ok((String::new(), vec![]));
            }
            get_wod_scores_for_edit(lid).await
        },
    );

    let todays_wods = Resource::new(|| (), |_| get_todays_wods());

    view! {
        <Suspense fallback=|| view! { <p class="loading">"Loading..."</p> }>
            {move || {
                let wod_data = resolved_wod.get().and_then(|r| r.ok()).flatten();
                let (existing_notes, scores) = existing_scores
                    .get()
                    .and_then(|r| r.ok())
                    .unwrap_or_default();
                let focus = focus_section_id.clone();

                if let Some((wod, sections, _movements)) = wod_data {
                    view! {
                        <WodScoreForm
                            wod=wod
                            sections=sections
                            focus_section=focus
                            existing_scores=scores
                            existing_notes=existing_notes
                            edit_log_id=edit_log_signal
                        />
                    }.into_any()
                } else {
                    let wods = todays_wods.get()
                        .and_then(|r| r.ok())
                        .unwrap_or_default();

                    if wods.is_empty() {
                        view! {
                            <div class="empty-state">
                                <p class="empty-title">"No WODs Today"</p>
                                <p class="empty-sub">"No workouts programmed for today. Use the Custom Log tab or check back later."</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="wod-picker">
                                <p class="picker-label">"Select a workout to log:"</p>
                                {wods.into_iter().map(|w| {
                                    let wid = w.id.clone();
                                    view! {
                                        <button
                                            class="wod-pick-card"
                                            on:click=move |_| selected_wod_id.set(wid.clone())
                                        >
                                            <span class="wod-pick-title">{w.title.clone()}</span>
                                            <span class="wod-pick-type">{w.workout_type.clone()}</span>
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }
            }}
        </Suspense>
    }
}
