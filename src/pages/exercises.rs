use crate::db::Exercise;
use leptos::prelude::*;

#[server]
async fn list_exercises() -> Result<Vec<Exercise>, ServerFnError> {
    let pool = crate::db::db().await?;
    crate::db::list_exercises_db(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn create_exercise(
    name: String,
    category: String,
    movement_type: String,
    description: String,
    demo_video_url: String,
) -> Result<(), ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let mt = if movement_type.is_empty() {
        None
    } else {
        Some(movement_type.as_str())
    };
    let desc = if description.is_empty() {
        None
    } else {
        Some(description.as_str())
    };
    let video = if demo_video_url.is_empty() {
        None
    } else {
        Some(demo_video_url.as_str())
    };
    crate::db::create_exercise_db(
        &pool,
        &name,
        &category,
        mt,
        &[],
        desc,
        video,
        Some(user_uuid),
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn update_exercise(
    id: String,
    name: String,
    category: String,
    movement_type: String,
    description: String,
    demo_video_url: String,
) -> Result<(), ServerFnError> {
    let _user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let mt = if movement_type.is_empty() {
        None
    } else {
        Some(movement_type.as_str())
    };
    let desc = if description.is_empty() {
        None
    } else {
        Some(description.as_str())
    };
    let video = if demo_video_url.is_empty() {
        None
    } else {
        Some(demo_video_url.as_str())
    };
    crate::db::update_exercise_db(&pool, uuid, &name, &category, mt, desc, video)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn delete_exercise(id: String) -> Result<(), ServerFnError> {
    let _user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::delete_exercise_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Convert a YouTube or Vimeo URL to its embed URL. Returns None for local/other URLs.
fn to_embed_url(url: &str) -> Option<String> {
    if url.contains("youtube.com/watch") {
        if let Some(pos) = url.find("v=") {
            let id = &url[pos + 2..];
            let id = id.split('&').next().unwrap_or(id);
            return Some(format!("https://www.youtube.com/embed/{}", id));
        }
    }
    if url.contains("youtu.be/") {
        if let Some(pos) = url.find("youtu.be/") {
            let id = &url[pos + 9..];
            let id = id.split('?').next().unwrap_or(id);
            return Some(format!("https://www.youtube.com/embed/{}", id));
        }
    }
    if url.contains("vimeo.com/") {
        if let Some(pos) = url.rfind('/') {
            let id = &url[pos + 1..];
            let id = id.split('?').next().unwrap_or(id);
            if id.chars().all(|c| c.is_ascii_digit()) {
                return Some(format!("https://player.vimeo.com/video/{}", id));
            }
        }
    }
    None
}

/// Single source of truth for exercise categories.
/// Each entry: (value, label, badge, css_class)
const CATEGORIES: &[(&str, &str, &str, &str)] = &[
    ("conditioning", "Conditioning", "CON", "badge--conditioning"),
    ("gymnastics", "Gymnastics", "GYM", "badge--gymnastics"),
    (
        "weightlifting",
        "Weightlifting",
        "WL",
        "badge--weightlifting",
    ),
    ("meditation", "Meditation", "MED", "badge--meditation"),
    ("sports", "Sports", "SPT", "badge--sports"),
];

fn category_badge(cat: &str) -> &'static str {
    CATEGORIES
        .iter()
        .find(|(v, _, _, _)| *v == cat)
        .map(|(_, _, b, _)| *b)
        .unwrap_or("GEN")
}

fn category_class(cat: &str) -> &'static str {
    CATEGORIES
        .iter()
        .find(|(v, _, _, _)| *v == cat)
        .map(|(_, _, _, c)| *c)
        .unwrap_or("")
}

fn category_options() -> impl IntoView {
    CATEGORIES
        .iter()
        .map(|(val, label, _, _)| {
            view! { <option value={*val}>{*label}</option> }
        })
        .collect_view()
}

#[component]
pub fn ExercisesPage() -> impl IntoView {
    use crate::auth::AuthUser;
    let is_authed = use_context::<AuthUser>().is_some();

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

    let active_filter = RwSignal::new("all".to_string());
    let show_form = RwSignal::new(false);
    let expanded_video = RwSignal::new(Option::<String>::None);
    let editing_exercise: RwSignal<Option<String>> = RwSignal::new(None);
    let show_delete = RwSignal::new(false);
    let pending_delete_id = RwSignal::new(String::new());

    let fab_view = if is_authed {
        view! {
            <button
                class={move || if show_form.get() { "fab fab--active" } else { "fab" }}
                on:click=move |_| show_form.update(|v| *v = !*v)
            >
                <span class="fab-icon"></span>
            </button>
        }
        .into_any()
    } else {
        ().into_view().into_any()
    };

    let form_view = move || {
        if show_form.get() {
            view! { <ExerciseForm create_action=create_action show_form=show_form/> }.into_any()
        } else {
            ().into_view().into_any()
        }
    };

    let list_view = view! {
        <Suspense fallback=|| view! { <p class="loading">"Loading movements..."</p> }>
            {move || {
                let filter = active_filter.get();
                exercises.get().map(|result| {
                    match result {
                        Ok(list) if list.is_empty() => {
                            view! {
                                <div class="empty-state">
                                    <p class="empty-title">"No movements yet"</p>
                                    <p class="empty-sub">"Tap + to build your library"</p>
                                </div>
                            }.into_any()
                        }
                        Ok(list) => {
                            let filtered: Vec<Exercise> = if filter == "all" {
                                list
                            } else {
                                list.into_iter().filter(|e| e.category == filter).collect()
                            };
                            view! {
                                <div class="exercise-grid">
                                    {filtered.into_iter().map(|ex| {
                                        view! {
                                            <ExerciseCard
                                                exercise=ex
                                                expanded_video=expanded_video
                                                editing_exercise=editing_exercise
                                                update_action=update_action
                                                pending_delete_id=pending_delete_id
                                                show_delete=show_delete
                                                is_authed=is_authed
                                            />
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! { <p class="error">{format!("Error: {}", e)}</p> }.into_any(),
                    }
                })
            }}
        </Suspense>
    }.into_any();

    let modal_view = view! {
        <DeleteModal
            show_delete=show_delete
            pending_delete_id=pending_delete_id
            delete_action=delete_action
        />
    }
    .into_any();

    let filter_view = view! {
        <FilterPills active_filter=active_filter/>
    }
    .into_any();

    view! {
        <div class="exercises-page">
            {fab_view}
            {filter_view}
            {form_view}
            {list_view}
            {modal_view}
        </div>
    }
}

#[component]
fn FilterPills(active_filter: RwSignal<String>) -> impl IntoView {
    view! {
        <div class="filter-pills">
            {std::iter::once(("all", "All"))
                .chain(CATEGORIES.iter().map(|(v, l, _, _)| (*v, *l)))
                .map(|(cat, label)| {
                let cat_active = cat.to_string();
                let cat_click = cat.to_string();
                view! {
                    <button
                        class="filter-pill"
                        class:active=move || active_filter.get() == cat_active
                        on:click=move |_| active_filter.set(cat_click.clone())
                    >
                        {label}
                    </button>
                }
            }).collect_view()}
        </div>
    }
}

#[component]
fn ExerciseForm(
    create_action: ServerAction<CreateExercise>,
    show_form: RwSignal<bool>,
) -> impl IntoView {
    let name_input = RwSignal::new(String::new());
    let category_input = RwSignal::new("conditioning".to_string());
    let movement_type_input = RwSignal::new(String::new());
    let description_input = RwSignal::new(String::new());
    let video_url = RwSignal::new(String::new());
    let uploading = RwSignal::new(false);
    let upload_error = RwSignal::new(String::new());
    let video_preview = RwSignal::new(String::new());
    let video_mode = RwSignal::new("url".to_string());
    let url_input = RwSignal::new(String::new());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let name = name_input.get_untracked();
        if name.is_empty() {
            return;
        }

        let reset_form = move || {
            name_input.set(String::new());
            description_input.set(String::new());
            video_url.set(String::new());
            video_preview.set(String::new());
            url_input.set(String::new());
            upload_error.set(String::new());
            show_form.set(false);
        };

        let mode = video_mode.get_untracked();

        if mode == "url" {
            let pasted_url = url_input.get_untracked();
            create_action.dispatch(CreateExercise {
                name,
                category: category_input.get_untracked(),
                movement_type: movement_type_input.get_untracked(),
                description: description_input.get_untracked(),
                demo_video_url: pasted_url,
            });
            reset_form();
            return;
        }

        #[cfg(feature = "hydrate")]
        {
            let has_file = !video_preview.get_untracked().is_empty();
            let already_uploaded = !video_url.get_untracked().is_empty();

            if !has_file {
                create_action.dispatch(CreateExercise {
                    name,
                    category: category_input.get_untracked(),
                    movement_type: movement_type_input.get_untracked(),
                    description: description_input.get_untracked(),
                    demo_video_url: String::new(),
                });
                reset_form();
                return;
            }

            if already_uploaded {
                create_action.dispatch(CreateExercise {
                    name,
                    category: category_input.get_untracked(),
                    movement_type: movement_type_input.get_untracked(),
                    description: description_input.get_untracked(),
                    demo_video_url: video_url.get_untracked(),
                });
                reset_form();
                return;
            }

            uploading.set(true);
            upload_error.set(String::new());
            let cat = category_input.get_untracked();
            let mt = movement_type_input.get_untracked();
            let desc = description_input.get_untracked();
            wasm_bindgen_futures::spawn_local(async move {
                match crate::voice::upload_video_file("exercise-video-input").await {
                    Ok(url_js) => {
                        let url = url_js.as_string().unwrap_or_default();
                        uploading.set(false);
                        create_action.dispatch(CreateExercise {
                            name,
                            category: cat,
                            movement_type: mt,
                            description: desc,
                            demo_video_url: url,
                        });
                        reset_form();
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
                name,
                category: category_input.get_untracked(),
                movement_type: movement_type_input.get_untracked(),
                description: description_input.get_untracked(),
                demo_video_url: url_input.get_untracked(),
            });
            reset_form();
        }
    };

    view! {
        <form class="exercise-form" on:submit=on_submit>
            <input
                type="text"
                placeholder="Exercise name"
                prop:value=move || name_input.get()
                on:input=move |ev| name_input.set(event_target_value(&ev))
            />
            <div class="form-row">
                <select
                    prop:value=move || category_input.get()
                    on:change=move |ev| category_input.set(event_target_value(&ev))
                >
                    {category_options()}
                </select>
                <input
                    type="text"
                    placeholder="Type (e.g. Olympic)"
                    prop:value=move || movement_type_input.get()
                    on:input=move |ev| movement_type_input.set(event_target_value(&ev))
                />
            </div>
            <input
                type="text"
                placeholder="Description (optional)"
                prop:value=move || description_input.get()
                on:input=move |ev| description_input.set(event_target_value(&ev))
            />

            <VideoUpload
                video_mode=video_mode
                url_input=url_input
                video_preview=video_preview
                video_url=video_url
                upload_error=upload_error
            />

            <button
                type="submit"
                class="form-submit"
                disabled=move || uploading.get() || create_action.pending().get()
            >
                {move || if uploading.get() {
                    view! { <span class="spinner"></span>" Uploading..." }.into_any()
                } else if create_action.pending().get() {
                    view! { <span class="spinner"></span>" Saving..." }.into_any()
                } else {
                    view! { "Add Movement" }.into_any()
                }}
            </button>
        </form>
    }
}

#[component]
fn VideoUpload(
    video_mode: RwSignal<String>,
    url_input: RwSignal<String>,
    video_preview: RwSignal<String>,
    video_url: RwSignal<String>,
    upload_error: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="video-upload">
            <div class="video-mode-toggle">
                <button
                    type="button"
                    class="video-mode-btn"
                    class:active=move || video_mode.get() == "url"
                    on:click=move |_| {
                        video_mode.set("url".to_string());
                        video_preview.set(String::new());
                        video_url.set(String::new());
                        upload_error.set(String::new());
                    }
                >
                    <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                        <path d="M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z"/>
                    </svg>
                    " URL"
                </button>
                <button
                    type="button"
                    class="video-mode-btn"
                    class:active=move || video_mode.get() == "file"
                    on:click=move |_| {
                        video_mode.set("file".to_string());
                        url_input.set(String::new());
                        upload_error.set(String::new());
                    }
                >
                    <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                        <path d="M16.5 6v11.5c0 2.21-1.79 4-4 4s-4-1.79-4-4V5c0-1.38 1.12-2.5 2.5-2.5s2.5 1.12 2.5 2.5v10.5c0 .55-.45 1-1 1s-1-.45-1-1V6H10v9.5c0 1.38 1.12 2.5 2.5 2.5s2.5-1.12 2.5-2.5V5c0-2.21-1.79-4-4-4S7 2.79 7 5v12.5c0 3.04 2.46 5.5 5.5 5.5s5.5-2.46 5.5-5.5V6h-1.5z"/>
                    </svg>
                    " File"
                </button>
            </div>

            {move || (video_mode.get() == "url").then(|| view! {
                <input
                    type="text"
                    class="video-url-input"
                    placeholder="Paste YouTube or Vimeo URL"
                    prop:value=move || url_input.get()
                    on:input=move |ev| url_input.set(event_target_value(&ev))
                />
            })}

            {move || (video_mode.get() == "file").then(|| view! {
                <div class="video-file-section">
                    <label class="video-upload-label" for="exercise-video-input">
                        <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                            <path d="M9 16h6v-6h4l-7-7-7 7h4v6zm-4 2h14v2H5v-2z"/>
                        </svg>
                        " Choose File"
                    </label>
                    <input
                        type="file"
                        id="exercise-video-input"
                        accept=".mp4,.webm,.mov,.avi,.m4v"
                        class="video-file-input"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            if !val.is_empty() {
                                let name = val.rsplit(['/', '\\']).next().unwrap_or(&val).to_string();
                                video_preview.set(name);
                                video_url.set(String::new());
                            }
                        }
                    />
                    {move || {
                        let preview = video_preview.get();
                        (!preview.is_empty()).then(|| view! {
                            <div class="video-selected">
                                <span class="video-filename">{preview}</span>
                                <button
                                    type="button"
                                    class="video-clear"
                                    on:click=move |_| {
                                        video_preview.set(String::new());
                                        video_url.set(String::new());
                                    }
                                >"x"</button>
                            </div>
                        })
                    }}
                </div>
            })}

            {move || {
                let err = upload_error.get();
                (!err.is_empty()).then(|| view! { <div class="video-error">{err}</div> })
            }}
        </div>
    }
}

#[component]
fn ExerciseCard(
    exercise: Exercise,
    expanded_video: RwSignal<Option<String>>,
    editing_exercise: RwSignal<Option<String>>,
    update_action: ServerAction<UpdateExercise>,
    pending_delete_id: RwSignal<String>,
    show_delete: RwSignal<bool>,
    is_authed: bool,
) -> impl IntoView {
    let id = exercise.id.clone();
    let cat = exercise.category.clone();
    let badge_text = category_badge(&cat);
    let badge_cls = category_class(&cat);
    let has_video = exercise.demo_video_url.is_some();
    let video_src = exercise.demo_video_url.clone().unwrap_or_default();
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
                                <select
                                    prop:value=move || edit_category.get()
                                    on:change=move |ev| edit_category.set(event_target_value(&ev))
                                >
                                    {category_options()}
                                </select>
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
                                                if is_expanded && is_playing.get() {
                                                    expanded_video.set(None);
                                                    is_playing.set(false);
                                                    autoplay.set(false);
                                                } else {
                                                    autoplay.set(true);
                                                    is_playing.set(true);
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
                                {is_authed.then(|| {
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
                                                allow="autoplay"
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

#[component]
fn DeleteModal(
    show_delete: RwSignal<bool>,
    pending_delete_id: RwSignal<String>,
    delete_action: ServerAction<DeleteExercise>,
) -> impl IntoView {
    view! {
        <div
            class="confirm-overlay"
            style=move || if show_delete.get() { "display:flex" } else { "display:none" }
            on:click=move |_| show_delete.set(false)
        >
            <div class="confirm-dialog" on:click=move |ev| { ev.stop_propagation(); }>
                <p class="confirm-msg">"Delete this movement?"</p>
                <p class="confirm-sub">"This cannot be undone."</p>
                <div class="confirm-actions">
                    <button
                        class="confirm-cancel-btn"
                        on:click=move |_| show_delete.set(false)
                    >"Cancel"</button>
                    <button
                        class="confirm-delete-btn"
                        on:click=move |_| {
                            delete_action.dispatch(DeleteExercise {
                                id: pending_delete_id.get_untracked(),
                            });
                            show_delete.set(false);
                        }
                    >"Delete"</button>
                </div>
            </div>
        </div>
    }
}
