use crate::components::SingleSelect;
use crate::db::Exercise;
use leptos::prelude::*;

use super::{
    category_badge, category_class, category_select_options, to_embed_url, UpdateExercise,
};

#[component]
pub fn ExerciseCard(
    exercise: Exercise,
    expanded_video: RwSignal<Option<String>>,
    editing_exercise: RwSignal<Option<String>>,
    update_action: ServerAction<UpdateExercise>,
    pending_delete_id: RwSignal<String>,
    show_delete: RwSignal<bool>,
    is_coach: bool,
) -> impl IntoView {
    let id = exercise.id.clone();
    let cat = exercise.category.clone();
    let badge_text = category_badge(&cat);
    let badge_cls = category_class(&cat);
    let has_video = exercise.demo_video_url.is_some();
    let video_src = exercise.demo_video_url.clone().unwrap_or_default();
    let is_embed = to_embed_url(&video_src).is_some();
    let autoplay = RwSignal::new(false);
    let is_playing = RwSignal::new(false);

    let card_id_play = exercise.id.clone();
    let card_id_play_icon = exercise.id.clone();
    let card_id_toggle = exercise.id.clone();
    let card_id_editing = exercise.id.clone();
    let card_id_edit_btn = exercise.id.clone();
    let card_id_submit = exercise.id.clone();

    // Edit form signals
    let edit_name = RwSignal::new(String::new());
    let edit_category = RwSignal::new(String::new());
    let edit_movement_type = RwSignal::new(String::new());
    let edit_description = RwSignal::new(String::new());
    let edit_video_url = RwSignal::new(String::new());

    // Init values for pre-populating
    let init_name = exercise.name.clone();
    let init_cat = exercise.category.clone();
    let init_mt = exercise.movement_type.clone().unwrap_or_default();
    let init_desc = exercise.description.clone().unwrap_or_default();
    let init_video = exercise.demo_video_url.clone().unwrap_or_default();

    view! {
        <div
            class=move || {
                if has_video {
                    "exercise-card exercise-card--has-video"
                } else {
                    "exercise-card"
                }
            }
            on:click=move |_| {
                if editing_exercise.get().is_some() { return; }
                if !has_video { return; }
                expanded_video.update(|v| {
                    if v.as_ref() == Some(&card_id_toggle) {
                        *v = None;
                        is_playing.set(false);
                        autoplay.set(false);
                    } else {
                        autoplay.set(false);
                        *v = Some(card_id_toggle.clone());
                    }
                });
            }
        >
            {
                let card_id_submit_c = card_id_submit.clone();
                let card_id_editing_c = card_id_editing.clone();
                let card_id_edit_btn_c = card_id_edit_btn.clone();
                let card_id_play_c = card_id_play.clone();
                let card_id_play_icon_c = card_id_play_icon.clone();
                let init_name_c2 = init_name.clone();
                let init_cat_c2 = init_cat.clone();
                let init_mt_c2 = init_mt.clone();
                let init_desc_c2 = init_desc.clone();
                let init_video_c2 = init_video.clone();
                let id_c2 = id.clone();
                let exercise_name_c = exercise.name.clone();
                let exercise_mt_c = exercise.movement_type.clone();
                let exercise_id_c = exercise.id.clone();
                let video_src_c2 = video_src.clone();
                let badge_text_c = badge_text;
                let badge_cls_c = badge_cls;
                move || {
                let eid = card_id_submit_c.clone();
                let eid_editing = card_id_editing_c.clone();
                let eid_edit_btn = card_id_edit_btn_c.clone();
                let cid_play = card_id_play_c.clone();
                let cid_play_icon = card_id_play_icon_c.clone();
                let iname = init_name_c2.clone();
                let icat = init_cat_c2.clone();
                let imt = init_mt_c2.clone();
                let idesc = init_desc_c2.clone();
                let ivideo = init_video_c2.clone();
                let id_del = id_c2.clone();
                let ex_name = exercise_name_c.clone();
                let ex_mt = exercise_mt_c.clone();
                let ex_id = exercise_id_c.clone();
                let v_src = video_src_c2.clone();
                let b_text = badge_text_c;
                let b_cls = badge_cls_c;
                if editing_exercise.get().as_ref() == Some(&eid_editing) {
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
                    let embed = to_embed_url(&v_src);
                    view! {
                        <div class="exercise-card-top">
                            <span class={format!("exercise-badge {}", b_cls)}>{b_text}</span>
                            <div class="exercise-card-actions" on:click=move |ev| ev.stop_propagation()>
                                {has_video.then(|| {
                                    let cid_p = cid_play.clone();

                                    let cid_pi = cid_play_icon.clone();
                                    view! {
                                        <button
                                            class=move || {
                                                if is_playing.get() {
                                                    "exercise-play exercise-play--active"
                                                } else {
                                                    "exercise-play"
                                                }
                                            }
                                            on:click=move |_| {
                                                let is_expanded = expanded_video.get().as_ref() == Some(&cid_p);
                                                if is_expanded {
                                                    expanded_video.set(None);
                                                    is_playing.set(false);
                                                    autoplay.set(false);
                                                } else {
                                                    autoplay.set(true);
                                                    if !is_embed {
                                                        is_playing.set(true);
                                                    }
                                                    expanded_video.set(Some(cid_p.clone()));
                                                }
                                            }
                                        >
                                            {move || {
                                                if is_playing.get() && expanded_video.get().as_ref() == Some(&cid_pi) {
                                                    view! {
                                                        <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
                                                            <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
                                                        </svg>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
                                                            <path d="M8 5v14l11-7z"/>
                                                        </svg>
                                                    }.into_any()
                                                }
                                            }}
                                        </button>
                                    }
                                })}
                                {is_coach.then(|| {
                                    let iname = iname.clone();
                                    let icat = icat.clone();
                                    let imt = imt.clone();
                                    let idesc = idesc.clone();
                                    let ivideo = ivideo.clone();
                                    let eid_edit_btn = eid_edit_btn.clone();
                                    let id_del = id_del.clone();
                                    view! {
                                        <button
                                            class="exercise-edit-btn"
                                            on:click=move |_| {
                                                edit_name.set(iname.clone());
                                                edit_category.set(icat.clone());
                                                edit_movement_type.set(imt.clone());
                                                edit_description.set(idesc.clone());
                                                edit_video_url.set(ivideo.clone());
                                                editing_exercise.set(Some(eid_edit_btn.clone()));
                                            }
                                        >"✎"</button>
                                        <button
                                            class="exercise-delete"
                                            on:click=move |_| {
                                                pending_delete_id.set(id_del.clone());
                                                show_delete.set(true);
                                            }
                                        >"×"</button>
                                    }
                                })}
                            </div>
                        </div>
                        <h3 class="exercise-name">{ex_name}</h3>
                        {ex_mt.map(|mt| view! {
                            <span class="exercise-type">{mt}</span>
                        })}
                        {
                            let vid_id = ex_id.clone();
                            let vid_src = v_src.clone();
                            move || {
                                let is_expanded = expanded_video.get().as_ref() == Some(&vid_id);
                                let should_autoplay = autoplay.get();
                                let embed_c = embed.clone();
                                let vid_src_c = vid_src.clone();
                                is_expanded.then(move || {
                                    if let Some(ref embed_url) = embed_c {
                                        let src = if should_autoplay {
                                            let sep = if embed_url.contains('?') { "&" } else { "?" };
                                            format!("{}{}autoplay=1", embed_url, sep)
                                        } else {
                                            embed_url.clone()
                                        };
                                        view! {
                                            <iframe
                                                class="exercise-video"
                                                src={src}
                                                allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                                                allowfullscreen=true
                                            />
                                        }.into_any()
                                    } else if should_autoplay {
                                            view! {
                                                <video
                                                    class="exercise-video"
                                                    src={vid_src_c.clone()}
                                                    controls
                                                    autoplay
                                                    playsinline
                                                    preload="metadata"
                                                    on:pause=move |_| is_playing.set(false)
                                                    on:ended=move |_| is_playing.set(false)
                                                    on:play=move |_| is_playing.set(true)
                                                />
                                            }.into_any()
                                        } else {
                                            view! {
                                                <video
                                                    class="exercise-video"
                                                    src={vid_src_c.clone()}
                                                    controls
                                                    playsinline
                                                    preload="metadata"
                                                    on:pause=move |_| is_playing.set(false)
                                                    on:ended=move |_| is_playing.set(false)
                                                    on:play=move |_| is_playing.set(true)
                                                />
                                            }.into_any()
                                    }
                                })
                            }
                        }
                    }.into_any()
                }
            }
            }
        </div>
    }
}
