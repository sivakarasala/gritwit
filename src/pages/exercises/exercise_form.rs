use crate::components::{SingleSelect, VideoUpload};
use leptos::prelude::*;

use super::{category_select_options, default_scoring_type, scoring_type_options, CreateExercise};

#[component]
pub fn ExerciseForm(
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
    let scoring_type_input = RwSignal::new(default_scoring_type("conditioning").to_string());

    // Auto-suggest scoring type when category changes
    Effect::new(move |_| {
        let cat = category_input.get();
        scoring_type_input.set(default_scoring_type(&cat).to_string());
    });

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
                scoring_type: scoring_type_input.get_untracked(),
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
                    scoring_type: scoring_type_input.get_untracked(),
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
                    scoring_type: scoring_type_input.get_untracked(),
                });
                reset_form();
                return;
            }

            uploading.set(true);
            upload_error.set(String::new());
            let cat = category_input.get_untracked();
            let mt = movement_type_input.get_untracked();
            let desc = description_input.get_untracked();
            let st = scoring_type_input.get_untracked();
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
                            scoring_type: st,
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
                scoring_type: scoring_type_input.get_untracked(),
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
                <SingleSelect
                    options=category_select_options()
                    selected=category_input
                    placeholder="Category"
                />
                <input
                    type="text"
                    placeholder="Type (e.g. Olympic)"
                    prop:value=move || movement_type_input.get()
                    on:input=move |ev| movement_type_input.set(event_target_value(&ev))
                />
            </div>
            <SingleSelect
                options=scoring_type_options()
                selected=scoring_type_input
                placeholder="Scoring type"
            />
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
