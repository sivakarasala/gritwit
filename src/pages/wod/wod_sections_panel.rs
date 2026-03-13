use crate::components::{SelectOption, SingleSelect};
use leptos::prelude::*;

use super::wod_section_card::WodSectionCard;
use super::{list_wod_sections, CreateWodSection, DeleteWodSection, UpdateWodSection};

fn phase_options() -> Vec<SelectOption> {
    vec![
        SelectOption {
            value: "warmup".into(),
            label: "Warm-Up".into(),
        },
        SelectOption {
            value: "strength".into(),
            label: "Strength".into(),
        },
        SelectOption {
            value: "conditioning".into(),
            label: "Conditioning".into(),
        },
        SelectOption {
            value: "cooldown".into(),
            label: "Cool Down".into(),
        },
        SelectOption {
            value: "optional".into(),
            label: "Optional".into(),
        },
        SelectOption {
            value: "personal".into(),
            label: "Personal".into(),
        },
    ]
}

fn section_type_options() -> Vec<SelectOption> {
    vec![
        SelectOption {
            value: "fortime".into(),
            label: "For Time".into(),
        },
        SelectOption {
            value: "amrap".into(),
            label: "AMRAP".into(),
        },
        SelectOption {
            value: "emom".into(),
            label: "EMOM".into(),
        },
        SelectOption {
            value: "strength".into(),
            label: "Strength".into(),
        },
        SelectOption {
            value: "static".into(),
            label: "Static".into(),
        },
    ]
}

#[component]
pub fn WodSectionsPanel(wod_id: String, is_coach: bool) -> impl IntoView {
    let add_section_action = ServerAction::<CreateWodSection>::new();
    let delete_section_action = ServerAction::<DeleteWodSection>::new();
    let update_section_action = ServerAction::<UpdateWodSection>::new();

    let wid = wod_id.clone();
    let sections = Resource::new(
        move || {
            (
                wid.clone(),
                add_section_action.version().get(),
                delete_section_action.version().get(),
                update_section_action.version().get(),
            )
        },
        |(id, _, _, _)| list_wod_sections(id),
    );

    let show_add_section = RwSignal::new(false);
    let phase_input = RwSignal::new("conditioning".to_string());
    let title_input = RwSignal::new(String::new());
    let type_input = RwSignal::new("fortime".to_string());
    let cap_input = RwSignal::new(String::new());
    let rounds_input = RwSignal::new(String::new());
    let notes_input = RwSignal::new(String::new());

    let wid_submit = wod_id.clone();

    view! {
        <div class="wod-sections" on:click=move |ev| ev.stop_propagation()>
            <Suspense fallback=|| view! { <span class="loading">"Loading sections..."</span> }>
                {move || sections.get().map(|res| match res {
                    Err(_) => view! {
                        <p class="error">"Could not load sections"</p>
                    }.into_any(),
                    Ok(secs) if secs.is_empty() => view! {
                        <p class="wod-sections-empty">"No sections yet"</p>
                    }.into_any(),
                    Ok(secs) => view! {
                        <div class="wod-section-list">
                            {secs.into_iter().map(|sec| {
                                view! {
                                    <WodSectionCard
                                        section=sec
                                        is_coach=is_coach
                                        delete_action=delete_section_action
                                        update_action=update_section_action
                                    />
                                }
                            }).collect_view()}
                        </div>
                    }.into_any(),
                })}
            </Suspense>

            {is_coach.then(move || {
                let wid_s = wid_submit.clone();
                view! {
                    <div class="wod-add-section">
                        {move || (!show_add_section.get()).then(|| view! {
                            <button
                                class="wod-add-section-btn"
                                on:click=move |_| show_add_section.set(true)
                            >"+ Add Section"</button>
                        })}
                        {move || show_add_section.get().then(|| {
                            let wid = wid_s.clone();
                            view! {
                                <form
                                    class="wod-section-form"
                                    on:submit=move |ev| {
                                        ev.prevent_default();
                                        add_section_action.dispatch(CreateWodSection {
                                            wod_id: wid.clone(),
                                            phase: phase_input.get_untracked(),
                                            title: title_input.get_untracked(),
                                            section_type: type_input.get_untracked(),
                                            time_cap_minutes: cap_input.get_untracked(),
                                            rounds: rounds_input.get_untracked(),
                                            notes: notes_input.get_untracked(),
                                        });
                                        title_input.set(String::new());
                                        cap_input.set(String::new());
                                        rounds_input.set(String::new());
                                        notes_input.set(String::new());
                                        show_add_section.set(false);
                                    }
                                >
                                    <div class="form-row">
                                        <SingleSelect options=phase_options() selected=phase_input placeholder="Phase" />
                                        <SingleSelect options=section_type_options() selected=type_input placeholder="Type" />
                                    </div>
                                    <input
                                        type="text"
                                        placeholder="Title (optional)"
                                        prop:value=move || title_input.get()
                                        on:input=move |ev| title_input.set(event_target_value(&ev))
                                    />
                                    <div class="form-row">
                                        <input
                                            type="number"
                                            placeholder="Time cap (min)"
                                            prop:value=move || cap_input.get()
                                            on:input=move |ev| cap_input.set(event_target_value(&ev))
                                        />
                                        <input
                                            type="number"
                                            placeholder="Rounds"
                                            prop:value=move || rounds_input.get()
                                            on:input=move |ev| rounds_input.set(event_target_value(&ev))
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
                                            disabled=move || add_section_action.pending().get()
                                        >
                                            {move || if add_section_action.pending().get() {
                                                view! { <span class="spinner"></span>" Adding..." }.into_any()
                                            } else {
                                                view! { "Add Section" }.into_any()
                                            }}
                                        </button>
                                        <button
                                            type="button"
                                            class="wod-cancel-btn"
                                            on:click=move |_| show_add_section.set(false)
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
