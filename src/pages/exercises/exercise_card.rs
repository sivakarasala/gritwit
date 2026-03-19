use crate::components::SingleSelect;
use crate::db::Exercise;
use leptos::prelude::*;

use super::{category_select_options, scoring_type_options, to_embed_url, UpdateExercise};

#[component]
pub fn ExerciseCard(
    exercise: Exercise,
    expanded_id: RwSignal<Option<String>>,
    editing_exercise: RwSignal<Option<String>>,
    update_action: ServerAction<UpdateExercise>,
    pending_delete_id: RwSignal<String>,
    show_delete: RwSignal<bool>,
    is_coach: bool,
    is_admin: bool,
    current_user_id: Option<String>,
) -> impl IntoView {
    let id = exercise.id.clone();
    let video_src = exercise.demo_video_url.clone().unwrap_or_default();
    let has_video = !video_src.is_empty();
    let description = exercise.description.clone().unwrap_or_default();
    let has_description = !description.is_empty();
    let can_delete = is_admin
        || current_user_id
            .as_deref()
            .zip(exercise.created_by.as_deref())
            .map(|(uid, owner)| uid == owner)
            .unwrap_or(false);

    // Edit form signals
    let edit_name = RwSignal::new(String::new());
    let edit_category = RwSignal::new(String::new());
    let edit_movement_type = RwSignal::new(String::new());
    let edit_description = RwSignal::new(String::new());
    let edit_video_url = RwSignal::new(String::new());
    let edit_scoring_type = RwSignal::new(String::new());

    let init_name = exercise.name.clone();
    let init_cat = exercise.category.clone();
    let init_mt = exercise.movement_type.clone().unwrap_or_default();
    let init_desc = description.clone();
    let init_video = video_src.clone();
    let init_scoring_type = exercise.scoring_type.clone();

    let id_toggle = id.clone();
    let id_edit_btn = id.clone();
    let id_submit = id.clone();
    let id_del = id.clone();

    view! {
        <div class="exercise-row">
            {move || {
                let eid = id_submit.clone();
                let eid_edit_btn = id_edit_btn.clone();
                let id_del = id_del.clone();
                let iname = init_name.clone();
                let icat = init_cat.clone();
                let imt = init_mt.clone();
                let idesc = init_desc.clone();
                let ivideo = init_video.clone();
                let iscoring = init_scoring_type.clone();

                if editing_exercise.get().as_ref() == Some(&eid) {
                    view! {
                        <form
                            class="exercise-edit-form"
                            on:click=move |ev| ev.stop_propagation()
                            on:submit=move |ev| {
                                ev.prevent_default();
                                let n = edit_name.get_untracked();
                                if n.is_empty() { return; }
                                update_action.dispatch(UpdateExercise {
                                    id: eid.clone(),
                                    name: n,
                                    category: edit_category.get_untracked(),
                                    movement_type: edit_movement_type.get_untracked(),
                                    description: edit_description.get_untracked(),
                                    demo_video_url: edit_video_url.get_untracked(),
                                    scoring_type: edit_scoring_type.get_untracked(),
                                });
                                editing_exercise.set(None);
                            }
                        >
                            <input
                                type="text"
                                placeholder="Exercise name"
                                prop:value=move || edit_name.get()
                                on:input=move |ev| edit_name.set(event_target_value(&ev))
                            />
                            <div class="form-row">
                                <SingleSelect
                                    options=category_select_options()
                                    selected=edit_category
                                    placeholder="Category"
                                />
                                <input
                                    type="text"
                                    placeholder="Type (e.g. Olympic)"
                                    prop:value=move || edit_movement_type.get()
                                    on:input=move |ev| edit_movement_type.set(event_target_value(&ev))
                                />
                            </div>
                            <SingleSelect
                                options=scoring_type_options()
                                selected=edit_scoring_type
                                placeholder="Scoring type"
                            />
                            <input
                                type="text"
                                placeholder="Description (optional)"
                                prop:value=move || edit_description.get()
                                on:input=move |ev| edit_description.set(event_target_value(&ev))
                            />
                            <input
                                type="text"
                                placeholder="Video URL (optional)"
                                prop:value=move || edit_video_url.get()
                                on:input=move |ev| edit_video_url.set(event_target_value(&ev))
                            />
                            <div class="exercise-edit-btns">
                                <button type="submit" class="form-submit">"Save"</button>
                                <button
                                    type="button"
                                    class="wod-cancel-btn"
                                    on:click=move |_| editing_exercise.set(None)
                                >"Cancel"</button>
                            </div>
                        </form>
                    }.into_any()
                } else {
                    let ex_name = exercise.name.clone();
                    let ex_mt = exercise.movement_type.clone();
                    let ex_id2 = exercise.id.clone();
                    let v_src = video_src.clone();
                    let embed = to_embed_url(&v_src);
                    let desc_text = description.clone();
                    let id_toggle_c = id_toggle.clone();

                    view! {
                        <div
                            class="exercise-row-header"
                            on:click=move |_| {
                                if editing_exercise.get().is_some() { return; }
                                expanded_id.update(|v| {
                                    if v.as_ref() == Some(&id_toggle_c) {
                                        *v = None;
                                    } else {
                                        *v = Some(id_toggle_c.clone());
                                    }
                                });
                            }
                        >
                            <div class="exercise-row-info">
                                <span class="exercise-name">{ex_name}</span>
                                {ex_mt.as_ref().map(|mt| view! {
                                    <span class="exercise-type">{mt.clone()}</span>
                                })}
                            </div>
                        </div>

                        // ── Expanded panel ──────────────────────────────────────
                        {move || {
                            let vid_src = v_src.clone();
                            let embed_c = embed.clone();
                            let desc_c = desc_text.clone();
                            let iname = iname.clone();
                            let icat = icat.clone();
                            let imt = imt.clone();
                            let idesc = idesc.clone();
                            let ivideo = ivideo.clone();
                            let iscoring = iscoring.clone();
                            let eid_edit = eid_edit_btn.clone();
                            let id_del = id_del.clone();

                            let is_expanded = expanded_id.get().as_ref() == Some(&ex_id2);
                            is_expanded.then(move || view! {
                                <div class="exercise-panel">
                                    {has_video.then(|| {
                                        let src = vid_src.clone();
                                        if let Some(ref embed_url) = embed_c {
                                            view! {
                                                <iframe
                                                    class="exercise-video"
                                                    src={embed_url.clone()}
                                                    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                                                    allowfullscreen=true
                                                />
                                            }.into_any()
                                        } else {
                                            view! {
                                                <video
                                                    class="exercise-video"
                                                    src={src}
                                                    controls
                                                    playsinline
                                                    preload="metadata"
                                                />
                                            }.into_any()
                                        }
                                    })}
                                    {has_description.then(|| view! {
                                        <p class="exercise-description">{desc_c}</p>
                                    })}
                                    {(!has_video && !has_description).then(|| view! {
                                        <p class="exercise-no-details">"No details added yet."</p>
                                    })}
                                    {is_coach.then(|| view! {
                                        <div class="exercise-panel-actions">
                                            <button
                                                class="exercise-edit-btn"
                                                on:click=move |_| {
                                                    edit_name.set(iname.clone());
                                                    edit_category.set(icat.clone());
                                                    edit_movement_type.set(imt.clone());
                                                    edit_description.set(idesc.clone());
                                                    edit_video_url.set(ivideo.clone());
                                                    edit_scoring_type.set(iscoring.clone());
                                                    editing_exercise.set(Some(eid_edit.clone()));
                                                }
                                            >"✎ Edit"</button>
                                            {can_delete.then(|| view! {
                                                <button
                                                    class="exercise-delete"
                                                    on:click=move |_| {
                                                        pending_delete_id.set(id_del.clone());
                                                        show_delete.set(true);
                                                    }
                                                >"× Delete"</button>
                                            })}
                                        </div>
                                    })}
                                </div>
                            })
                        }}
                    }.into_any()
                }
            }}
        </div>
    }
}
