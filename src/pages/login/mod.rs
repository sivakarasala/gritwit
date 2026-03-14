use crate::auth::password::{LoginWithPassword, RegisterWithPassword};
use leptos::prelude::*;

fn clean_error(e: &ServerFnError) -> String {
    let raw = e.to_string();
    raw.strip_prefix("error running server function: ")
        .or_else(|| raw.strip_prefix("ServerFnError: "))
        .unwrap_or(&raw)
        .to_string()
}

#[derive(Clone)]
struct ToastMsg {
    message: String,
    is_error: bool,
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let login = ServerAction::<LoginWithPassword>::new();
    let register = ServerAction::<RegisterWithPassword>::new();
    let (show_register, set_show_register) = signal(false);
    let toast: RwSignal<Option<ToastMsg>> = RwSignal::new(None);

    let login_pending = login.pending();
    let register_pending = register.pending();

    let register_form: NodeRef<leptos::html::Form> = NodeRef::new();

    Effect::new(move |_| match login.value().get() {
        Some(Ok(_)) => {
            #[cfg(feature = "hydrate")]
            let _ = js_sys::eval("window.location.href = '/'");
        }
        Some(Err(e)) => {
            let msg = clean_error(&e);
            toast.set(None);
            leptos::task::spawn_local(async move {
                toast.set(Some(ToastMsg {
                    message: msg,
                    is_error: true,
                }));
            });
        }
        None => {}
    });

    Effect::new(move |_| match register.value().get() {
        Some(Ok(_)) => {
            if let Some(form) = register_form.get() {
                form.reset();
            }
            #[cfg(feature = "hydrate")]
            let _ = js_sys::eval("window.location.href = '/'");
        }
        Some(Err(e)) => {
            let msg = clean_error(&e);
            toast.set(None);
            leptos::task::spawn_local(async move {
                toast.set(Some(ToastMsg {
                    message: msg,
                    is_error: true,
                }));
            });
        }
        None => {}
    });

    view! {
        <div class="login-page">
            <div class="login-card">
                <a href="/auth/google/login" rel="external" class="google-btn">
                    <span class="google-icon">
                        <svg viewBox="0 0 24 24" width="18" height="18">
                            <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z"/>
                            <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                            <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
                            <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
                        </svg>
                    </span>
                    "Sign in with Google"
                </a>

                <div class="auth-divider"><span>"or"</span></div>

                <Show when=move || !show_register.get()>
                    <div class="auth-form">
                        <ActionForm action=login>
                            <input type="email" name="email" placeholder="Email" required />
                            <input type="password" name="password" placeholder="Password" required />
                            <button
                                type="submit"
                                class="auth-submit"
                                disabled=move || login_pending.get()
                            >
                                <Show when=move || login_pending.get()>
                                    <span class="auth-spinner"></span>
                                </Show>
                                {move || if login_pending.get() { "Signing in..." } else { "Sign in" }}
                            </button>
                        </ActionForm>
                    </div>
                    <p class="auth-switch">
                        "No account? "
                        <button class="auth-link" on:click=move |_| set_show_register.set(true)>
                            "Register"
                        </button>
                    </p>
                </Show>

                <Show when=move || show_register.get()>
                    <div class="auth-form">
                        <ActionForm action=register node_ref=register_form>
                            <input type="text" name="name" placeholder="Name" required maxlength="100" />
                            <input type="email" name="email" placeholder="Email" required />
                            <input type="password" name="password" placeholder="Password (min 8 chars)" required minlength="8" />
                            <button
                                type="submit"
                                class="auth-submit"
                                disabled=move || register_pending.get()
                            >
                                <Show when=move || register_pending.get()>
                                    <span class="auth-spinner"></span>
                                </Show>
                                {move || if register_pending.get() { "Creating account..." } else { "Create account" }}
                            </button>
                        </ActionForm>
                    </div>
                    <p class="auth-switch">
                        "Have an account? "
                        <button class="auth-link" on:click=move |_| set_show_register.set(false)>
                            "Sign in"
                        </button>
                    </p>
                </Show>

                <p class="login-hint">
                    "Or browse "
                    <a href="/wod">"WOD"</a>
                    " and "
                    <a href="/exercises">"Exercises"</a>
                    " without signing in"
                </p>
            </div>
        </div>

        <Show when=move || toast.get().is_some()>
            {move || toast.get().map(|t| {
                let cls = if t.is_error { "toast toast--error" } else { "toast toast--success" };
                view! {
                    <div class=cls>
                        <span class="toast__msg">{t.message}</span>
                        <button class="toast__close" on:click=move |_| toast.set(None)>"×"</button>
                    </div>
                }
            })}
        </Show>
    }
}
