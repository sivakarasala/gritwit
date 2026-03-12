use crate::db::Exercise;
use leptos::prelude::*;

#[server]
async fn list_exercises() -> Result<Vec<Exercise>, ServerFnError> {
    let _user = crate::auth::session::require_auth().await?;
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
    crate::db::create_exercise_db(&pool, &name, &category, mt, &[], desc, video, Some(user_uuid))
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
    // YouTube: youtube.com/watch?v=ID or youtu.be/ID
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
    // Vimeo: vimeo.com/ID
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

fn category_badge(cat: &str) -> &'static str {
    match cat {
        "crossfit" => "CF",
        "strength" => "STR",
        "meditation" => "MED",
        "breathing" => "BRE",
        _ => "GEN",
    }
}

fn category_class(cat: &str) -> &'static str {
    match cat {
        "crossfit" => "badge--crossfit",
        "strength" => "badge--strength",
        "meditation" => "badge--meditation",
        "breathing" => "badge--breathing",
        _ => "",
    }
}

#[component]
pub fn ExercisesPage() -> impl IntoView {
    let create_action = ServerAction::<CreateExercise>::new();
    let delete_action = ServerAction::<DeleteExercise>::new();

    let exercises = Resource::new(
        move || (create_action.version().get(), delete_action.version().get()),
        |_| list_exercises(),
    );

    let name_input = RwSignal::new(String::new());
    let category_input = RwSignal::new("crossfit".to_string());
    let movement_type_input = RwSignal::new(String::new());
    let description_input = RwSignal::new(String::new());
    let active_filter = RwSignal::new("all".to_string());
    let show_form = RwSignal::new(false);
    let video_url = RwSignal::new(String::new());
    let uploading = RwSignal::new(false);
    let upload_error = RwSignal::new(String::new());
    let video_preview = RwSignal::new(String::new());
    let video_mode = RwSignal::new("url".to_string()); // "url" or "file"
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

        // URL mode — use the pasted link directly
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

        // File mode
        #[cfg(feature = "hydrate")]
        {
            let has_file = !video_preview.get_untracked().is_empty();
            let already_uploaded = !video_url.get_untracked().is_empty();

            if !has_file {
                // No file selected
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

            // File selected but not uploaded — upload then create
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

    // Expanded video signal for exercise cards
    let expanded_video = RwSignal::new(Option::<String>::None);

    // Delete confirmation
    let show_delete = RwSignal::new(false);
    let pending_delete_id = RwSignal::new(String::new());

    view! {
        <div class="exercises-page">
            <button
                class={move || if show_form.get() { "fab fab--active" } else { "fab" }}
                on:click=move |_| show_form.update(|v| *v = !*v)
            >
                <span class="fab-icon"></span>
            </button>

            // Filter pills
            <div class="filter-pills">
                {["all", "crossfit", "strength", "meditation", "breathing"].into_iter().map(|cat| {
                    let cat_str = cat.to_string();
                    let label = match cat {
                        "all" => "All",
                        "crossfit" => "CrossFit",
                        "strength" => "Strength",
                        "meditation" => "Meditation",
                        "breathing" => "Breathing",
                        _ => cat,
                    };
                    let cat_active = cat_str.clone();
                    let cat_click = cat_str.clone();
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

            // Collapsible add form
            {move || show_form.get().then(|| view! {
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
                            <option value="crossfit">"CrossFit"</option>
                            <option value="strength">"Strength"</option>
                            <option value="meditation">"Meditation"</option>
                            <option value="breathing">"Breathing"</option>
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

                    // Video source toggle
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
                                <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
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
                                <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
                                    <path d="M16.5 6v11.5c0 2.21-1.79 4-4 4s-4-1.79-4-4V5c0-1.38 1.12-2.5 2.5-2.5s2.5 1.12 2.5 2.5v10.5c0 .55-.45 1-1 1s-1-.45-1-1V6H10v9.5c0 1.38 1.12 2.5 2.5 2.5s2.5-1.12 2.5-2.5V5c0-2.21-1.79-4-4-4S7 2.79 7 5v12.5c0 3.04 2.46 5.5 5.5 5.5s5.5-2.46 5.5-5.5V6h-1.5z"/>
                                </svg>
                                " File"
                            </button>
                        </div>

                        // URL input
                        {move || (video_mode.get() == "url").then(|| view! {
                            <input
                                type="text"
                                class="video-url-input"
                                placeholder="Paste YouTube or Vimeo URL"
                                prop:value=move || url_input.get()
                                on:input=move |ev| url_input.set(event_target_value(&ev))
                            />
                        })}

                        // File input
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
                                    accept="video/*"
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

                    <button type="submit" class="form-submit" disabled=move || uploading.get()>
                        {move || if uploading.get() { "Uploading..." } else { "Add Movement" }}
                    </button>
                </form>
            })}

            <Suspense fallback=|| view! { <p class="loading">"Loading movements..."</p> }>
                {move || {
                    let filter = active_filter.get();
                    exercises.get().map(|result| {
                        match result {
                            Ok(list) if list.is_empty() => {
                                view! {
                                    <div class="empty-state">
                                        <p class="empty-title">"No movements yet"</p>
                                        <p class="empty-sub">"Tap + Add to build your library"</p>
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
                                            let id = ex.id.clone();
                                            let cat = ex.category.clone();
                                            let badge_text = category_badge(&cat);
                                            let badge_cls = category_class(&cat);
                                            let has_video = ex.demo_video_url.is_some();
                                            let video_src = ex.demo_video_url.clone().unwrap_or_default();
                                            let card_id = ex.id.clone();
                                            view! {
                                                <div class="exercise-card">
                                                    <div class="exercise-card-top">
                                                        <span class={format!("exercise-badge {}", badge_cls)}>{badge_text}</span>
                                                        <div class="exercise-card-actions">
                                                            {has_video.then(|| {
                                                                let vid = card_id.clone();
                                                                view! {
                                                                    <button
                                                                        class="exercise-play"
                                                                        on:click=move |_| {
                                                                            expanded_video.update(|v| {
                                                                                if v.as_ref() == Some(&vid) {
                                                                                    *v = None;
                                                                                } else {
                                                                                    *v = Some(vid.clone());
                                                                                }
                                                                            });
                                                                        }
                                                                    >
                                                                        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
                                                                            <path d="M8 5v14l11-7z"/>
                                                                        </svg>
                                                                    </button>
                                                                }
                                                            })}
                                                            <button
                                                                class="exercise-delete"
                                                                on:click={
                                                                    let id = id.clone();
                                                                    move |_| {
                                                                        pending_delete_id.set(id.clone());
                                                                        show_delete.set(true);
                                                                    }
                                                                }
                                                            >"×"</button>
                                                        </div>
                                                    </div>
                                                    <h3 class="exercise-name">{ex.name}</h3>
                                                    {ex.movement_type.map(|mt| view! {
                                                        <span class="exercise-type">{mt}</span>
                                                    })}
                                                    // Video player (expanded)
                                                    {
                                                        let vid_id = ex.id.clone();
                                                        let vid_src = video_src.clone();
                                                        let embed = to_embed_url(&vid_src);
                                                        move || {
                                                            let is_expanded = expanded_video.get().as_ref() == Some(&vid_id);
                                                            is_expanded.then(|| {
                                                                if let Some(ref embed_url) = embed {
                                                                    view! {
                                                                        <iframe
                                                                            class="exercise-video"
                                                                            src={embed_url.clone()}
                                                                        />
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <video
                                                                            class="exercise-video"
                                                                            src={vid_src.clone()}
                                                                            controls
                                                                            playsinline
                                                                            preload="metadata"
                                                                        />
                                                                    }.into_any()
                                                                }
                                                            })
                                                        }
                                                    }
                                                </div>
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

            // Delete confirmation modal — always in DOM, shown/hidden via style
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
        </div>
    }
}
