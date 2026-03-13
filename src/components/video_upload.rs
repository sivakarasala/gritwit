use leptos::prelude::*;

/// Reusable video upload component with URL paste and file upload modes.
#[component]
pub fn VideoUpload(
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
