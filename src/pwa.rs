use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub fn register_service_worker() {
    let global = js_sys::global();
    let navigator = js_sys::Reflect::get(&global, &JsValue::from_str("navigator"))
        .unwrap_or(JsValue::UNDEFINED);

    let sw_container = js_sys::Reflect::get(&navigator, &JsValue::from_str("serviceWorker"))
        .unwrap_or(JsValue::UNDEFINED);
    if sw_container.is_undefined() {
        return;
    }

    wasm_bindgen_futures::spawn_local(async move {
        let register_fn = match js_sys::Reflect::get(&sw_container, &JsValue::from_str("register"))
        {
            Ok(f) if f.is_function() => f.unchecked_into::<js_sys::Function>(),
            _ => return,
        };

        let promise = match register_fn.call1(&sw_container, &JsValue::from_str("/sw.js")) {
            Ok(p) => p.unchecked_into::<js_sys::Promise>(),
            Err(_) => return,
        };

        match JsFuture::from(promise).await {
            Ok(_) => {
                let _ = log_to_console("Service worker registered");
            }
            Err(_) => {
                let _ = log_to_console("Service worker registration failed");
            }
        }
    });
}

fn log_to_console(msg: &str) -> Result<(), JsValue> {
    let global = js_sys::global();
    let console = js_sys::Reflect::get(&global, &JsValue::from_str("console"))?;
    let log_fn = js_sys::Reflect::get(&console, &JsValue::from_str("log"))?
        .unchecked_into::<js_sys::Function>();
    log_fn.call1(&console, &JsValue::from_str(msg))?;
    Ok(())
}
