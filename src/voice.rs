use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
export function speech_recognition_supported() {
    return !!(window.SpeechRecognition || window.webkitSpeechRecognition);
}

export function start_speech(on_result, on_error, on_end) {
    const SR = window.SpeechRecognition || window.webkitSpeechRecognition;
    if (!SR) { on_error('Speech recognition not supported'); return null; }
    const rec = new SR();
    rec.continuous = false;
    rec.interimResults = false;
    rec.lang = 'en-US';
    rec.onresult = (e) => {
        const t = e.results[0][0].transcript;
        on_result(t);
    };
    rec.onerror = (e) => on_error(e.error);
    rec.onend = () => on_end();
    rec.start();
    return rec;
}

export function stop_speech(rec) {
    if (rec) rec.stop();
}

export function toggle_theme() {
    const html = document.documentElement;
    const current = html.getAttribute('data-theme') || 'dark';
    const next = current === 'dark' ? 'light' : 'dark';
    html.setAttribute('data-theme', next);
    localStorage.setItem('theme', next);
    return next;
}

export async function upload_video_file(file_input_id) {
    const input = document.getElementById(file_input_id);
    if (!input || !input.files || !input.files[0]) {
        return '';
    }
    const file = input.files[0];
    if (!file.type.startsWith('video/')) {
        throw new Error('Only video files are allowed');
    }
    if (file.size > 50 * 1024 * 1024) {
        throw new Error('Video must be under 50MB');
    }
    const form = new FormData();
    form.append('video', file);
    const resp = await fetch('/api/v1/upload/video', { method: 'POST', body: form });
    if (!resp.ok) {
        const text = await resp.text();
        throw new Error(text || 'Upload failed');
    }
    const data = await resp.json();
    return data.url;
}
")]
extern "C" {
    pub fn speech_recognition_supported() -> bool;

    pub fn start_speech(
        on_result: &Closure<dyn Fn(String)>,
        on_error: &Closure<dyn Fn(String)>,
        on_end: &Closure<dyn Fn()>,
    ) -> JsValue;

    pub fn stop_speech(rec: &JsValue);

    pub fn toggle_theme() -> String;

    #[wasm_bindgen(catch)]
    pub async fn upload_video_file(file_input_id: &str) -> Result<JsValue, JsValue>;
}
