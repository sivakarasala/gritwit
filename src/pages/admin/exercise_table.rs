use crate::components::{DeleteModal, VideoUpload};
use crate::db::Exercise;
use crate::pages::exercises::{
    category_select_options, default_scoring_type, list_exercises, scoring_type_options,
    CreateExercise, DeleteExercise, UpdateExercise,
};
use leptos::prelude::*;

// ── Toast ─────────────────────────────────────────────────

#[derive(Clone)]
struct ToastMsg {
    message: String,
    is_error: bool,
}

// ── Add Exercise Modal ────────────────────────────────────

#[component]
fn AddExerciseModal(
    show: RwSignal<bool>,
    create_action: ServerAction<CreateExercise>,
    modal_error: RwSignal<Option<String>>,
) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let category = RwSignal::new("conditioning".to_string());
    let movement_type = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let video_mode = RwSignal::new("url".to_string());
    let url_input = RwSignal::new(String::new());
    let video_preview = RwSignal::new(String::new());
    let video_url = RwSignal::new(String::new());
    let upload_error = RwSignal::new(String::new());
    let uploading = RwSignal::new(false);
    let scoring_type = RwSignal::new(default_scoring_type("conditioning").to_string());

    Effect::new(move |_| {
        let cat = category.get();
        scoring_type.set(default_scoring_type(&cat).to_string());
    });

    let close = move || {
        show.set(false);
        name.set(String::new());
        category.set("conditioning".to_string());
        movement_type.set(String::new());
        description.set(String::new());
        video_mode.set("url".to_string());
        url_input.set(String::new());
        video_preview.set(String::new());
        video_url.set(String::new());
        upload_error.set(String::new());
        modal_error.set(None);
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let n = name.get_untracked();
        if n.is_empty() {
            return;
        }

        let mode = video_mode.get_untracked();

        if mode == "url" {
            create_action.dispatch(CreateExercise {
                name: n,
                category: category.get_untracked(),
                movement_type: movement_type.get_untracked(),
                description: description.get_untracked(),
                demo_video_url: url_input.get_untracked(),
                scoring_type: scoring_type.get_untracked(),
            });
            close();
            return;
        }

        #[cfg(feature = "hydrate")]
        {
            let has_file = !video_preview.get_untracked().is_empty();
            let already_uploaded = !video_url.get_untracked().is_empty();

            if !has_file || already_uploaded {
                create_action.dispatch(CreateExercise {
                    name: n,
                    category: category.get_untracked(),
                    movement_type: movement_type.get_untracked(),
                    description: description.get_untracked(),
                    demo_video_url: video_url.get_untracked(),
                    scoring_type: scoring_type.get_untracked(),
                });
                close();
                return;
            }

            uploading.set(true);
            upload_error.set(String::new());
            let cat = category.get_untracked();
            let mt = movement_type.get_untracked();
            let desc = description.get_untracked();
            let st = scoring_type.get_untracked();
            wasm_bindgen_futures::spawn_local(async move {
                match crate::voice::upload_video_file("exercise-video-input").await {
                    Ok(url_js) => {
                        let url = url_js.as_string().unwrap_or_default();
                        uploading.set(false);
                        create_action.dispatch(CreateExercise {
                            name: n,
                            category: cat,
                            movement_type: mt,
                            description: desc,
                            demo_video_url: url,
                            scoring_type: st,
                        });
                        close();
                    }
                    Err(e) => {
                        uploading.set(false);
                        let msg = e
                            .as_string()
                            .or_else(|| {
                                js_sys::Reflect::get(&e, &"message".into())
                                    .ok()
                                    .and_then(|v| v.as_string())
                            })
                            .unwrap_or_else(|| "Upload failed".into());
                        upload_error.set(msg);
                    }
                }
            });
            return;
        }

        #[cfg(not(feature = "hydrate"))]
        {
            create_action.dispatch(CreateExercise {
                name: n,
                category: category.get_untracked(),
                movement_type: movement_type.get_untracked(),
                description: description.get_untracked(),
                demo_video_url: url_input.get_untracked(),
                scoring_type: scoring_type.get_untracked(),
            });
            close();
        }
    };

    view! {
        <div
            class="ex-modal-overlay"
            style=move || if show.get() { "display:flex" } else { "display:none" }
            on:click=move |_| close()
        >
            <div class="ex-modal" on:click=move |ev| ev.stop_propagation()>
                <div class="ex-modal-header">
                    <span class="ex-modal-title">"Add Exercise"</span>
                    <button class="ex-modal-close" on:click=move |_| close()>"×"</button>
                </div>
                <form class="ex-modal-form" on:submit=on_submit>
                    <div class="ex-modal-field">
                        <label>"Name"</label>
                        <input type="text" class="ex-modal-input" placeholder="e.g. Back Squat"
                            prop:value=move || name.get()
                            on:input=move |ev| name.set(event_target_value(&ev))
                            autofocus=true
                        />
                    </div>
                    <div class="ex-modal-row">
                        <div class="ex-modal-field">
                            <label>"Category"</label>
                            <select class="ex-modal-select"
                                prop:value=move || category.get()
                                on:change=move |ev| category.set(event_target_value(&ev))
                            >
                                {category_select_options().into_iter().map(|opt| view! {
                                    <option value={opt.value}>{opt.label}</option>
                                }).collect_view()}
                            </select>
                        </div>
                        <div class="ex-modal-field">
                            <label>"Movement Type"</label>
                            <input type="text" class="ex-modal-input" placeholder="e.g. Olympic"
                                prop:value=move || movement_type.get()
                                on:input=move |ev| movement_type.set(event_target_value(&ev))
                            />
                        </div>
                    </div>
                    <div class="ex-modal-field">
                        <label>"Scoring Type"</label>
                        <select class="ex-modal-select"
                            prop:value=move || scoring_type.get()
                            on:change=move |ev| scoring_type.set(event_target_value(&ev))
                        >
                            {scoring_type_options().into_iter().map(|opt| view! {
                                <option value={opt.value}>{opt.label}</option>
                            }).collect_view()}
                        </select>
                    </div>
                    <div class="ex-modal-field">
                        <label>"Description"</label>
                        <textarea class="ex-modal-textarea" placeholder="Instructions or notes (optional)"
                            prop:value=move || description.get()
                            on:input=move |ev| description.set(event_target_value(&ev))
                            rows="3"
                        ></textarea>
                    </div>
                    <div class="ex-modal-field">
                        <label>"Video"</label>
                        <VideoUpload
                            video_mode=video_mode
                            url_input=url_input
                            video_preview=video_preview
                            video_url=video_url
                            upload_error=upload_error
                        />
                    </div>
                    {move || modal_error.get().map(|err| view! {
                        <p class="ex-modal-error">{err}</p>
                    })}
                    <div class="ex-modal-actions">
                        <button type="button" class="ex-modal-btn ex-modal-btn--cancel"
                            on:click=move |_| close()
                        >"Cancel"</button>
                        <button type="submit" class="ex-modal-btn ex-modal-btn--save"
                            disabled=move || uploading.get() || create_action.pending().get()
                        >
                            {move || if uploading.get() {
                                "Uploading…"
                            } else if create_action.pending().get() {
                                "Saving…"
                            } else {
                                "Add Exercise"
                            }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}

// ── Exercise Table ────────────────────────────────────────

#[component]
pub fn ExerciseTable() -> impl IntoView {
    let create_action = ServerAction::<CreateExercise>::new();
    let delete_action = ServerAction::<DeleteExercise>::new();
    let update_action = ServerAction::<UpdateExercise>::new();

    let exercises = Resource::new(
        move || {
            (
                create_action.version().get(),
                delete_action.version().get(),
                update_action.version().get(),
            )
        },
        |_| list_exercises(),
    );

    let editing_id: RwSignal<Option<String>> = RwSignal::new(None);
    let show_add = RwSignal::new(false);
    let show_delete = RwSignal::new(false);
    let pending_delete_id = RwSignal::new(String::new());
    let toast: RwSignal<Option<ToastMsg>> = RwSignal::new(None);
    let modal_error: RwSignal<Option<String>> = RwSignal::new(None);

    // Toast on create
    Effect::new(move |_| {
        if let Some(result) = create_action.value().get() {
            match result {
                Ok(_) => toast.set(Some(ToastMsg {
                    message: "Exercise added!".into(),
                    is_error: false,
                })),
                Err(e) => {
                    let msg = e.to_string();
                    modal_error.set(Some(msg.clone()));
                    toast.set(Some(ToastMsg {
                        message: msg,
                        is_error: true,
                    }));
                }
            }
        }
    });

    // Toast on update
    Effect::new(move |_| {
        if let Some(result) = update_action.value().get() {
            match result {
                Ok(_) => toast.set(Some(ToastMsg {
                    message: "Exercise updated!".into(),
                    is_error: false,
                })),
                Err(e) => toast.set(Some(ToastMsg {
                    message: e.to_string(),
                    is_error: true,
                })),
            }
        }
    });

    // Toast on delete
    Effect::new(move |_| {
        if let Some(result) = delete_action.value().get() {
            match result {
                Ok(_) => toast.set(Some(ToastMsg {
                    message: "Exercise deleted.".into(),
                    is_error: false,
                })),
                Err(e) => toast.set(Some(ToastMsg {
                    message: e.to_string(),
                    is_error: true,
                })),
            }
        }
    });

    view! {
        <div class="admin-exercises">
            <div class="admin-exercises-mobile-msg">
                <p>"This section is only available on desktop."</p>
            </div>
            <div class="admin-exercises-content">
                <div class="admin-exercises-toolbar">
                    <button class="admin-add-btn"
                        on:click=move |_| show_add.set(true)
                    >"+ Add Exercise"</button>
                </div>

                <div class="admin-table-wrap">
                    <table class="admin-table">
                        <thead>
                            <tr>
                                <th>"Name"</th>
                                <th>"Category"</th>
                                <th>"Type"</th>
                                <th>"Scoring"</th>
                                <th>"Description"</th>
                                <th>"Video URL"</th>
                                <th>"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Suspense fallback=|| view! {
                                <tr><td colspan="7" class="admin-table-loading">"Loading…"</td></tr>
                            }>
                                {move || exercises.get().map(|result| match result {
                                    Ok(list) if list.is_empty() => view! {
                                        <tr><td colspan="7" class="admin-table-empty">"No exercises yet."</td></tr>
                                    }.into_any(),
                                    Ok(list) => list.into_iter().map(|ex| view! {
                                        <ExerciseRow
                                            exercise=ex
                                            editing_id=editing_id
                                            update_action=update_action
                                            pending_delete_id=pending_delete_id
                                            show_delete=show_delete
                                        />
                                    }).collect_view().into_any(),
                                    Err(e) => view! {
                                        <tr><td colspan="7" class="error">{format!("Error: {e}")}</td></tr>
                                    }.into_any(),
                                })}
                            </Suspense>
                        </tbody>
                    </table>
                </div>
            </div>

            <AddExerciseModal show=show_add create_action=create_action modal_error=modal_error/>

            <DeleteModal
                show=show_delete
                title="Delete this exercise?"
                subtitle="This cannot be undone."
                confirm_label="Delete"
                on_confirm=move || {
                    delete_action.dispatch(DeleteExercise {
                        id: pending_delete_id.get_untracked(),
                    });
                }
            />

            {move || toast.get().map(|t| {
                let cls = if t.is_error { "toast toast--error" } else { "toast toast--success" };
                view! {
                    <div class={cls}>
                        <span class="toast__msg">{t.message}</span>
                        <button class="toast__close" on:click=move |_| toast.set(None)>"×"</button>
                    </div>
                }
            })}
        </div>
    }
}

// ── Exercise Row ──────────────────────────────────────────

#[component]
fn ExerciseRow(
    exercise: Exercise,
    editing_id: RwSignal<Option<String>>,
    update_action: ServerAction<UpdateExercise>,
    pending_delete_id: RwSignal<String>,
    show_delete: RwSignal<bool>,
) -> impl IntoView {
    let id = exercise.id.clone();
    let init_name = exercise.name.clone();
    let init_category = exercise.category.clone();
    let init_mt = exercise.movement_type.clone().unwrap_or_default();
    let init_desc = exercise.description.clone().unwrap_or_default();
    let init_video = exercise.demo_video_url.clone().unwrap_or_default();
    let init_scoring = exercise.scoring_type.clone();

    let edit_name = RwSignal::new(String::new());
    let edit_category = RwSignal::new(String::new());
    let edit_movement_type = RwSignal::new(String::new());
    let edit_description = RwSignal::new(String::new());
    let edit_video_url = RwSignal::new(String::new());
    let edit_scoring_type = RwSignal::new(String::new());

    view! {
        {move || {
            let eid = id.clone();
            let iname = init_name.clone();
            let icat = init_category.clone();
            let imt = init_mt.clone();
            let idesc = init_desc.clone();
            let ivideo = init_video.clone();
            let iscoring = init_scoring.clone();

            if editing_id.get().as_deref() == Some(eid.as_str()) {
                let eid_save = eid.clone();
                view! {
                    <tr class="admin-table-edit-row">
                        <td>
                            <input type="text" class="admin-cell-input"
                                prop:value=move || edit_name.get()
                                on:input=move |ev| edit_name.set(event_target_value(&ev))
                            />
                        </td>
                        <td>
                            <select class="admin-cell-select"
                                prop:value=move || edit_category.get()
                                on:change=move |ev| edit_category.set(event_target_value(&ev))
                            >
                                {category_select_options().into_iter().map(|opt| view! {
                                    <option value={opt.value}>{opt.label}</option>
                                }).collect_view()}
                            </select>
                        </td>
                        <td>
                            <input type="text" class="admin-cell-input"
                                prop:value=move || edit_movement_type.get()
                                on:input=move |ev| edit_movement_type.set(event_target_value(&ev))
                            />
                        </td>
                        <td>
                            <select class="admin-cell-select"
                                prop:value=move || edit_scoring_type.get()
                                on:change=move |ev| edit_scoring_type.set(event_target_value(&ev))
                            >
                                {scoring_type_options().into_iter().map(|opt| view! {
                                    <option value={opt.value}>{opt.label}</option>
                                }).collect_view()}
                            </select>
                        </td>
                        <td>
                            <input type="text" class="admin-cell-input"
                                prop:value=move || edit_description.get()
                                on:input=move |ev| edit_description.set(event_target_value(&ev))
                            />
                        </td>
                        <td>
                            <input type="text" class="admin-cell-input"
                                prop:value=move || edit_video_url.get()
                                on:input=move |ev| edit_video_url.set(event_target_value(&ev))
                            />
                        </td>
                        <td class="admin-table-actions">
                            <button class="admin-btn admin-btn--save"
                                disabled=move || update_action.pending().get()
                                on:click=move |_| {
                                    let name = edit_name.get_untracked();
                                    if name.is_empty() { return; }
                                    update_action.dispatch(UpdateExercise {
                                        id: eid_save.clone(),
                                        name,
                                        category: edit_category.get_untracked(),
                                        movement_type: edit_movement_type.get_untracked(),
                                        description: edit_description.get_untracked(),
                                        demo_video_url: edit_video_url.get_untracked(),
                                        scoring_type: edit_scoring_type.get_untracked(),
                                    });
                                    editing_id.set(None);
                                }
                            >"Save"</button>
                            <button class="admin-btn admin-btn--cancel"
                                on:click=move |_| editing_id.set(None)
                            >"Cancel"</button>
                        </td>
                    </tr>
                }.into_any()
            } else {
                let iname2 = iname.clone();
                let icat2 = icat.clone();
                let imt2 = imt.clone();
                let idesc2 = idesc.clone();
                let ivideo2 = ivideo.clone();
                let iscoring2 = iscoring.clone();
                let eid_edit = eid.clone();
                let eid_del = eid.clone();
                view! {
                    <tr class="admin-table-row">
                        <td class="admin-cell-name">{iname}</td>
                        <td>{icat}</td>
                        <td>{imt}</td>
                        <td>{iscoring}</td>
                        <td class="admin-cell-desc">{idesc}</td>
                        <td class="admin-cell-video">{ivideo}</td>
                        <td class="admin-table-actions">
                            <button class="admin-btn admin-btn--edit"
                                on:click=move |_| {
                                    edit_name.set(iname2.clone());
                                    edit_category.set(icat2.clone());
                                    edit_movement_type.set(imt2.clone());
                                    edit_description.set(idesc2.clone());
                                    edit_video_url.set(ivideo2.clone());
                                    edit_scoring_type.set(iscoring2.clone());
                                    editing_id.set(Some(eid_edit.clone()));
                                }
                            >"Edit"</button>
                            <button class="admin-btn admin-btn--delete"
                                on:click=move |_| {
                                    pending_delete_id.set(eid_del.clone());
                                    show_delete.set(true);
                                }
                            >"Delete"</button>
                        </td>
                    </tr>
                }.into_any()
            }
        }}
    }
}
