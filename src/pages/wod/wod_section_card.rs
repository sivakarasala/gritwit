use crate::components::{SelectOption, SingleSelect};
use crate::db::WodSection;
use leptos::prelude::*;

use super::section_movements_panel::SectionMovementsPanel;
use super::{phase_class, phase_label, section_type_label, DeleteWodSection, UpdateWodSection};

#[component]
pub fn WodSectionCard(
    section: WodSection,
    is_coach: bool,
    delete_action: ServerAction<DeleteWodSection>,
    update_action: ServerAction<UpdateWodSection>,
) -> impl IntoView {
    let editing = RwSignal::new(false);
    let confirm_delete = RwSignal::new(false);

    let edit_phase = RwSignal::new(section.phase.clone());
    let edit_title = RwSignal::new(section.title.clone().unwrap_or_default());
    let edit_type = RwSignal::new(section.section_type.clone());
    let edit_cap = RwSignal::new(
        section
            .time_cap_minutes
            .map(|v| v.to_string())
            .unwrap_or_default(),
    );
    let edit_rounds = RwSignal::new(section.rounds.map(|v| v.to_string()).unwrap_or_default());
    let edit_notes = RwSignal::new(section.notes.clone().unwrap_or_default());

    let sec_id = section.id.clone();
    let sec_id_del = section.id.clone();
    let sec_id_submit = section.id.clone();
    let section_id_for_panel = section.id.clone();
    let sec_id_log = section.id.clone();

    let p_label = phase_label(&section.phase);
    let p_class = format!("phase-badge {}", phase_class(&section.phase));
    let t_label = section_type_label(&section.section_type);
    let cap_display = section.time_cap_minutes;
    let rounds_display = section.rounds;
    let title_display = section.title.clone();
    let notes_display = section.notes.clone();

    view! {
        <div class="wod-section-card">
            {move || if editing.get() {
                let sid = sec_id_submit.clone();
                view! {
                    <form
                        class="wod-section-edit-form"
                        on:submit=move |ev| {
                            ev.prevent_default();
                            update_action.dispatch(UpdateWodSection {
                                id: sid.clone(),
                                phase: edit_phase.get_untracked(),
                                title: edit_title.get_untracked(),
                                section_type: edit_type.get_untracked(),
                                time_cap_minutes: edit_cap.get_untracked(),
                                rounds: edit_rounds.get_untracked(),
                                notes: edit_notes.get_untracked(),
                            });
                            editing.set(false);
                        }
                    >
                        <div class="form-row">
                            <SingleSelect
                                options=vec![
                                    SelectOption { value: "warmup".into(),       label: "Warm-Up".into() },
                                    SelectOption { value: "strength".into(),     label: "Strength".into() },
                                    SelectOption { value: "conditioning".into(), label: "Conditioning".into() },
                                    SelectOption { value: "cooldown".into(),     label: "Cool Down".into() },
                                    SelectOption { value: "optional".into(),     label: "Optional".into() },
                                    SelectOption { value: "personal".into(),     label: "Personal".into() },
                                ]
                                selected=edit_phase
                                placeholder="Phase"
                            />
                            <SingleSelect
                                options=vec![
                                    SelectOption { value: "fortime".into(),  label: "For Time".into() },
                                    SelectOption { value: "amrap".into(),    label: "AMRAP".into() },
                                    SelectOption { value: "emom".into(),     label: "EMOM".into() },
                                    SelectOption { value: "strength".into(), label: "Strength".into() },
                                    SelectOption { value: "static".into(),   label: "Static".into() },
                                ]
                                selected=edit_type
                                placeholder="Type"
                            />
                        </div>
                        <input
                            type="text"
                            placeholder="Title (optional)"
                            prop:value=move || edit_title.get()
                            on:input=move |ev| edit_title.set(event_target_value(&ev))
                        />
                        <div class="form-row">
                            <input
                                type="number"
                                placeholder="Time cap (min)"
                                prop:value=move || edit_cap.get()
                                on:input=move |ev| edit_cap.set(event_target_value(&ev))
                            />
                            <input
                                type="number"
                                placeholder="Rounds"
                                prop:value=move || edit_rounds.get()
                                on:input=move |ev| edit_rounds.set(event_target_value(&ev))
                            />
                        </div>
                        <input
                            type="text"
                            placeholder="Notes (optional)"
                            prop:value=move || edit_notes.get()
                            on:input=move |ev| edit_notes.set(event_target_value(&ev))
                        />
                        <div class="wod-mov-form-btns">
                            <button
                                type="submit"
                                class="form-submit"
                                disabled=move || update_action.pending().get()
                            >
                                {move || if update_action.pending().get() {
                                    view! { <span class="spinner"></span>" Saving..." }.into_any()
                                } else {
                                    view! { "Save" }.into_any()
                                }}
                            </button>
                            <button
                                type="button"
                                class="wod-cancel-btn"
                                on:click=move |_| editing.set(false)
                            >"Cancel"</button>
                        </div>
                    </form>
                }.into_any()
            } else {
                let p_class_c = p_class.clone();
                let t_label_c = t_label;
                let title_c = title_display.clone();
                let notes_c = notes_display.clone();
                let sec_id_e = sec_id.clone();
                let sec_id_d = sec_id_del.clone();
                view! {
                    <div class="wod-section-header">
                        <span class={p_class_c}>{p_label}</span>
                        {(!t_label_c.is_empty()).then(|| view! {
                            <span class="section-type-label">{t_label_c}</span>
                        })}
                        {cap_display.map(|c| view! {
                            <span class="section-cap">{"⏱ "}{c}{" min"}</span>
                        })}
                        {rounds_display.map(|r| view! {
                            <span class="section-rounds">{r}{" rounds"}</span>
                        })}
                        {title_c.map(|t| view! {
                            <span class="section-title-label">{t}</span>
                        })}
                        {is_coach.then(move || {
                            let eid = sec_id_e.clone();
                            let did = sec_id_d.clone();
                            view! {
                                <div class="section-actions">
                                    <button
                                        class="wod-mov-edit"
                                        on:click=move |_| editing.set(true)
                                    >"✎"</button>
                                    {move || if confirm_delete.get() {
                                        let did2 = did.clone();
                                        view! {
                                            <span class="inline-confirm">
                                                "Delete? "
                                                <button
                                                    type="button"
                                                    class="confirm-delete-btn"
                                                    on:click=move |_| {
                                                        delete_action.dispatch(DeleteWodSection { id: did2.clone() });
                                                        confirm_delete.set(false);
                                                    }
                                                >"Yes"</button>
                                                " "
                                                <button
                                                    type="button"
                                                    class="confirm-cancel-btn"
                                                    on:click=move |_| confirm_delete.set(false)
                                                >"No"</button>
                                            </span>
                                        }.into_any()
                                    } else {
                                        let eid2 = eid.clone();
                                        let _ = eid2;
                                        view! {
                                            <button
                                                class="wod-mov-delete"
                                                on:click=move |_| confirm_delete.set(true)
                                            >"×"</button>
                                        }.into_any()
                                    }}
                                </div>
                            }
                        })}
                    </div>
                    {notes_c.map(|n| view! {
                        <p class="section-notes">{n}</p>
                    })}
                    <SectionMovementsPanel
                        section_id=section_id_for_panel.clone()
                        is_coach=is_coach
                    />
                    <div class="section-log-footer">
                        <button
                            class="section-log-btn"
                            on:click={
                                let url = format!("/log?section_id={}", sec_id_log);
                                move |_| {
                                    let navigate = leptos_router::hooks::use_navigate();
                                    navigate(&url, Default::default());
                                }
                            }
                        >"Log Result"</button>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
