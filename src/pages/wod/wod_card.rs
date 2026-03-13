use crate::db::Wod;
use leptos::prelude::*;

use super::wod_sections_panel::WodSectionsPanel;
use super::{wod_type_class, wod_type_label, UpdateWod};

#[component]
pub fn WodCard(
    wod: Wod,
    is_coach: bool,
    editing_wod: RwSignal<Option<String>>,
    update_action: ServerAction<UpdateWod>,
    pending_delete_wod_id: RwSignal<String>,
    show_delete_wod: RwSignal<bool>,
) -> impl IntoView {
    let expanded = RwSignal::new(true);
    let wid_del = wod.id.clone();
    let wid_panel = StoredValue::new(wod.id.clone());
    let wid_edit = wod.id.clone();
    let wid_editing = wod.id.clone();
    let wid_submit = wod.id.clone();

    let edit_title = RwSignal::new(String::new());
    let edit_desc = RwSignal::new(String::new());
    let edit_type = RwSignal::new(String::new());
    let edit_cap = RwSignal::new(String::new());
    let edit_date = RwSignal::new(String::new());

    let type_label = wod_type_label(&wod.workout_type);
    let type_cls = format!("wod-badge {}", wod_type_class(&wod.workout_type));
    let cap = wod.time_cap_minutes;
    let title = wod.title.clone();
    let desc = wod.description.clone();
    let date = wod.programmed_date.clone();

    let init_title = wod.title.clone();
    let init_desc = wod.description.clone().unwrap_or_default();
    let init_type = wod.workout_type.clone();
    let init_cap = wod
        .time_cap_minutes
        .map(|c| c.to_string())
        .unwrap_or_default();
    let init_date = wod.programmed_date.clone();

    view! {
        <div
            class="wod-card"
            on:click=move |_| {
                if editing_wod.get().is_some() { return; }
                expanded.update(|v| *v = !*v);
            }
        >
            {move || {
                if editing_wod.get().as_ref() == Some(&wid_editing) {
                    let wid_s = wid_submit.clone();
                    view! {
                        <form
                            class="wod-edit-form"
                            on:click=move |ev| ev.stop_propagation()
                            on:submit=move |ev| {
                                ev.prevent_default();
                                let t = edit_title.get_untracked();
                                if t.is_empty() { return; }
                                update_action.dispatch(UpdateWod {
                                    id: wid_s.clone(),
                                    title: t,
                                    description: edit_desc.get_untracked(),
                                    workout_type: edit_type.get_untracked(),
                                    time_cap_minutes: edit_cap.get_untracked(),
                                    programmed_date: edit_date.get_untracked(),
                                });
                                editing_wod.set(None);
                            }
                        >
                            <div class="form-row">
                                <input
                                    type="date"
                                    prop:value=move || edit_date.get()
                                    on:input=move |ev| edit_date.set(event_target_value(&ev))
                                />
                                <select
                                    prop:value=move || edit_type.get()
                                    on:change=move |ev| edit_type.set(event_target_value(&ev))
                                >
                                    <option value="fortime">"For Time"</option>
                                    <option value="amrap">"AMRAP"</option>
                                    <option value="emom">"EMOM"</option>
                                    <option value="tabata">"Tabata"</option>
                                    <option value="strength">"Strength"</option>
                                    <option value="custom">"Custom"</option>
                                </select>
                            </div>
                            <input
                                type="text"
                                placeholder="Title"
                                prop:value=move || edit_title.get()
                                on:input=move |ev| edit_title.set(event_target_value(&ev))
                            />
                            <input
                                type="text"
                                placeholder="Description (optional)"
                                prop:value=move || edit_desc.get()
                                on:input=move |ev| edit_desc.set(event_target_value(&ev))
                            />
                            <input
                                type="number"
                                placeholder="Time cap (minutes)"
                                prop:value=move || edit_cap.get()
                                on:input=move |ev| edit_cap.set(event_target_value(&ev))
                            />
                            <div class="wod-edit-btns">
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
                                    on:click=move |_| editing_wod.set(None)
                                >"Cancel"</button>
                            </div>
                        </form>
                    }.into_any()
                } else {
                    let init_title_c = init_title.clone();
                    let init_desc_c = init_desc.clone();
                    let init_type_c = init_type.clone();
                    let init_cap_c = init_cap.clone();
                    let init_date_c = init_date.clone();
                    let wid_e = wid_edit.clone();
                    let wid_d = wid_del.clone();
                    let title_c = title.clone();
                    let desc_c = desc.clone();
                    let date_c = date.clone();
                    let type_cls_c = type_cls.clone();
                    view! {
                        <div class="wod-card-top">
                            <div class="wod-card-meta">
                                <span class={type_cls_c}>{type_label}</span>
                                <span class="wod-date">{date_c}</span>
                            </div>
                            <div class="wod-card-actions" on:click=move |ev| ev.stop_propagation()>
                                {is_coach.then(move || view! {
                                    <button
                                        class="wod-edit-btn"
                                        on:click=move |_| {
                                            edit_title.set(init_title_c.clone());
                                            edit_desc.set(init_desc_c.clone());
                                            edit_type.set(init_type_c.clone());
                                            edit_cap.set(init_cap_c.clone());
                                            edit_date.set(init_date_c.clone());
                                            editing_wod.set(Some(wid_e.clone()));
                                        }
                                    >"✎"</button>
                                    <button
                                        class="wod-delete"
                                        on:click=move |_| {
                                            pending_delete_wod_id.set(wid_d.clone());
                                            show_delete_wod.set(true);
                                        }
                                    >"×"</button>
                                })}
                            </div>
                        </div>
                        <h2 class="wod-title">{title_c}</h2>
                        {desc_c.map(|d| view! {
                            <p class="wod-desc">{d}</p>
                        })}
                        {cap.map(|c| view! {
                            <span class="wod-timecap">"⏱ "{c}" min"</span>
                        })}
                        {move || expanded.get().then(|| view! {
                            <WodSectionsPanel
                                wod_id=wid_panel.get_value()
                                is_coach=is_coach
                            />
                        })}
                    }.into_any()
                }
            }}
        </div>
    }
}
