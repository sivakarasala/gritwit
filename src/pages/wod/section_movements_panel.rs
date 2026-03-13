use crate::components::{SelectOption, SingleSelect};
use leptos::prelude::*;

use super::{
    get_section_movements, list_exercises_for_wod, AddSectionMovement, DeleteSectionMovement,
    UpdateSectionMovement,
};

#[component]
pub fn SectionMovementsPanel(section_id: String, is_coach: bool) -> impl IntoView {
    let add_action = ServerAction::<AddSectionMovement>::new();
    let update_action = ServerAction::<UpdateSectionMovement>::new();
    let delete_action = ServerAction::<DeleteSectionMovement>::new();

    let sid = section_id.clone();
    let movements = Resource::new(
        move || {
            (
                sid.clone(),
                add_action.version().get(),
                update_action.version().get(),
                delete_action.version().get(),
            )
        },
        |(id, _, _, _)| get_section_movements(id),
    );

    let exercises = Resource::new(|| (), |_| list_exercises_for_wod());

    let show_add = RwSignal::new(false);
    let ex_id = RwSignal::new(String::new());
    let rep_scheme_input = RwSignal::new(String::new());
    let weight_male_input = RwSignal::new(String::new());
    let weight_female_input = RwSignal::new(String::new());
    let notes_input = RwSignal::new(String::new());

    let editing_mov: RwSignal<Option<String>> = RwSignal::new(None);
    let edit_ex_id = RwSignal::new(String::new());
    let edit_rep_scheme = RwSignal::new(String::new());
    let edit_weight_male = RwSignal::new(String::new());
    let edit_weight_female = RwSignal::new(String::new());
    let edit_notes = RwSignal::new(String::new());

    let confirm_delete_id: RwSignal<Option<String>> = RwSignal::new(None);

    let sid_submit = section_id.clone();

    view! {
        <div class="section-movements">
            <Suspense fallback=|| view! { <span class="loading">"Loading..."</span> }>
                {move || movements.get().map(|res| match res {
                    Err(_) => view! {
                        <p class="error">"Could not load movements"</p>
                    }.into_any(),
                    Ok(movs) if movs.is_empty() => view! {
                        <p class="wod-mov-empty">"No movements yet"</p>
                    }.into_any(),
                    Ok(movs) => view! {
                        <ol class="wod-mov-list">
                            {movs.into_iter().map(|m| {
                                // Use StoredValue so the movement ID can be freely accessed
                                // from multiple closures without move-consuming it.
                                let mid_sv = StoredValue::new(m.id.clone());
                                let m_ex_id = m.exercise_id.clone();
                                let m_rep = m.rep_scheme.clone().unwrap_or_default();
                                let m_male = m.weight_kg_male.map(|v| v.to_string()).unwrap_or_default();
                                let m_female = m.weight_kg_female.map(|v| v.to_string()).unwrap_or_default();
                                let m_notes = m.notes.clone().unwrap_or_default();

                                let m_detail = {
                                    let mut parts = Vec::new();
                                    if let Some(r) = &m.rep_scheme { parts.push(r.clone()); }
                                    match (m.weight_kg_male, m.weight_kg_female) {
                                        (Some(male), Some(female)) => parts.push(format!("{}/{}", male, female)),
                                        (Some(male), None) => parts.push(format!("{}", male)),
                                        (None, Some(female)) => parts.push(format!("{}", female)),
                                        (None, None) => {}
                                    }
                                    if parts.is_empty() { None } else { Some(parts.join(" - ")) }
                                };

                                view! {
                                    <li class="wod-mov-item">
                                        {move || {
                                            let mid_s = mid_sv.get_value();
                                            if editing_mov.get().as_ref() == Some(&mid_sv.get_value()) {
                                                view! {
                                                    <form
                                                        class="wod-mov-edit-form"
                                                        on:submit=move |ev| {
                                                            ev.prevent_default();
                                                            if edit_ex_id.get_untracked().is_empty() { return; }
                                                            update_action.dispatch(UpdateSectionMovement {
                                                                id: mid_s.clone(),
                                                                exercise_id: edit_ex_id.get_untracked(),
                                                                rep_scheme: edit_rep_scheme.get_untracked(),
                                                                weight_kg_male: edit_weight_male.get_untracked(),
                                                                weight_kg_female: edit_weight_female.get_untracked(),
                                                                notes: edit_notes.get_untracked(),
                                                            });
                                                            editing_mov.set(None);
                                                        }
                                                    >
                                                        <Suspense fallback=|| view! { <span>"..."</span> }>
                                                            {move || exercises.get().map(|res| match res {
                                                                Ok(exs) => {
                                                                    let options = exs.into_iter()
                                                                        .map(|(id, name)| SelectOption { value: id, label: name })
                                                                        .collect::<Vec<_>>();
                                                                    view! {
                                                                        <SingleSelect options=options selected=edit_ex_id placeholder="Select exercise" />
                                                                    }.into_any()
                                                                },
                                                                Err(_) => view! {
                                                                    <span>"Failed to load"</span>
                                                                }.into_any(),
                                                            })}
                                                        </Suspense>
                                                        <input
                                                            type="text"
                                                            placeholder="Rep scheme (e.g. 21-15-9)"
                                                            prop:value=move || edit_rep_scheme.get()
                                                            on:input=move |ev| edit_rep_scheme.set(event_target_value(&ev))
                                                        />
                                                        <div class="form-row">
                                                            <input
                                                                type="number"
                                                                placeholder="M kg"
                                                                step="0.5"
                                                                prop:value=move || edit_weight_male.get()
                                                                on:input=move |ev| edit_weight_male.set(event_target_value(&ev))
                                                            />
                                                            <input
                                                                type="number"
                                                                placeholder="F kg"
                                                                step="0.5"
                                                                prop:value=move || edit_weight_female.get()
                                                                on:input=move |ev| edit_weight_female.set(event_target_value(&ev))
                                                            />
                                                        </div>
                                                        <input
                                                            type="text"
                                                            placeholder="Notes"
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
                                                                on:click=move |_| editing_mov.set(None)
                                                            >"Cancel"</button>
                                                        </div>
                                                    </form>
                                                }.into_any()
                                            } else {
                                                let m_ex_id_c = m_ex_id.clone();
                                                let rep_c = m_rep.clone();
                                                let male_c = m_male.clone();
                                                let female_c = m_female.clone();
                                                let notes_c = m_notes.clone();
                                                let ex_name = m.exercise_name.clone();
                                                let detail_c = m_detail.clone();
                                                view! {
                                                    <div class="wod-mov-info">
                                                        <span class="wod-mov-name">{ex_name}</span>
                                                        {detail_c.map(|d| view! {
                                                            <span class="wod-mov-detail">{d}</span>
                                                        })}
                                                    </div>
                                                    {is_coach.then(move || view! {
                                                        <button
                                                            class="wod-mov-edit"
                                                            on:click=move |_| {
                                                                edit_ex_id.set(m_ex_id_c.clone());
                                                                edit_rep_scheme.set(rep_c.clone());
                                                                edit_weight_male.set(male_c.clone());
                                                                edit_weight_female.set(female_c.clone());
                                                                edit_notes.set(notes_c.clone());
                                                                editing_mov.set(Some(mid_sv.get_value()));
                                                            }
                                                        >"✎"</button>
                                                        {move || if confirm_delete_id.get().as_deref() == Some(mid_sv.get_value().as_str()) {
                                                            view! {
                                                                <span class="inline-confirm">
                                                                    "Remove? "
                                                                    <button
                                                                        type="button"
                                                                        class="confirm-delete-btn"
                                                                        on:click=move |_| {
                                                                            delete_action.dispatch(DeleteSectionMovement { id: mid_sv.get_value() });
                                                                            confirm_delete_id.set(None);
                                                                        }
                                                                    >"Yes"</button>
                                                                    " "
                                                                    <button
                                                                        type="button"
                                                                        class="confirm-cancel-btn"
                                                                        on:click=move |_| confirm_delete_id.set(None)
                                                                    >"No"</button>
                                                                </span>
                                                            }.into_any()
                                                        } else {
                                                            view! {
                                                                <button
                                                                    class="wod-mov-delete"
                                                                    on:click=move |_| confirm_delete_id.set(Some(mid_sv.get_value()))
                                                                >"×"</button>
                                                            }.into_any()
                                                        }}
                                                    })}
                                                }.into_any()
                                            }
                                        }}
                                    </li>
                                }
                            }).collect_view()}
                        </ol>
                    }.into_any(),
                })}
            </Suspense>

            {is_coach.then(move || {
                let sid = sid_submit.clone();
                view! {
                    <div class="wod-add-movement">
                        {move || (!show_add.get()).then(|| view! {
                            <button
                                class="wod-add-mov-btn"
                                on:click=move |_| show_add.set(true)
                            >"+ Add Movement"</button>
                        })}
                        {move || show_add.get().then(|| {
                            let sid2 = sid.clone();
                            view! {
                                <form
                                    class="wod-mov-form"
                                    on:submit=move |ev| {
                                        ev.prevent_default();
                                        if ex_id.get_untracked().is_empty() { return; }
                                        add_action.dispatch(AddSectionMovement {
                                            section_id: sid2.clone(),
                                            exercise_id: ex_id.get_untracked(),
                                            rep_scheme: rep_scheme_input.get_untracked(),
                                            weight_kg_male: weight_male_input.get_untracked(),
                                            weight_kg_female: weight_female_input.get_untracked(),
                                            notes: notes_input.get_untracked(),
                                        });
                                        ex_id.set(String::new());
                                        rep_scheme_input.set(String::new());
                                        weight_male_input.set(String::new());
                                        weight_female_input.set(String::new());
                                        notes_input.set(String::new());
                                        show_add.set(false);
                                    }
                                >
                                    <Suspense fallback=|| view! { <span>"Loading..."</span> }>
                                        {move || exercises.get().map(|res| match res {
                                            Ok(exs) => {
                                                let options = exs.into_iter()
                                                    .map(|(id, name)| SelectOption { value: id, label: name })
                                                    .collect::<Vec<_>>();
                                                view! {
                                                    <SingleSelect options=options selected=ex_id placeholder="Select exercise" />
                                                }.into_any()
                                            },
                                            Err(_) => view! {
                                                <span>"Failed to load exercises"</span>
                                            }.into_any(),
                                        })}
                                    </Suspense>
                                    <input
                                        type="text"
                                        placeholder="Rep scheme (e.g. 21-15-9)"
                                        prop:value=move || rep_scheme_input.get()
                                        on:input=move |ev| rep_scheme_input.set(event_target_value(&ev))
                                    />
                                    <div class="form-row">
                                        <input
                                            type="number"
                                            placeholder="M kg"
                                            step="0.5"
                                            prop:value=move || weight_male_input.get()
                                            on:input=move |ev| weight_male_input.set(event_target_value(&ev))
                                        />
                                        <input
                                            type="number"
                                            placeholder="F kg"
                                            step="0.5"
                                            prop:value=move || weight_female_input.get()
                                            on:input=move |ev| weight_female_input.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <input
                                        type="text"
                                        placeholder="Notes (optional)"
                                        prop:value=move || notes_input.get()
                                        on:input=move |ev| notes_input.set(event_target_value(&ev))
                                    />
                                    <div class="wod-mov-form-btns">
                                        <button
                                            type="submit"
                                            class="form-submit"
                                            disabled=move || add_action.pending().get()
                                        >
                                            {move || if add_action.pending().get() {
                                                view! { <span class="spinner"></span>" Adding..." }.into_any()
                                            } else {
                                                view! { "Add" }.into_any()
                                            }}
                                        </button>
                                        <button
                                            type="button"
                                            class="wod-cancel-btn"
                                            on:click=move |_| show_add.set(false)
                                        >"Cancel"</button>
                                    </div>
                                </form>
                            }
                        })}
                    </div>
                }
            })}
        </div>
    }
}
