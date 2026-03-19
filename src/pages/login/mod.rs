use crate::auth::clean_error;
use crate::auth::otp::{SendOtp, VerifyOtp};
use crate::auth::password::{LoginWithPassword, RegisterWithPassword};
use leptos::prelude::*;

/// Strip non-digits from phone input.
fn digits_only(raw: &str) -> String {
    raw.chars().filter(|c| c.is_ascii_digit()).collect()
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
    let send_otp = ServerAction::<SendOtp>::new();
    let verify_otp = ServerAction::<VerifyOtp>::new();

    let (show_register, set_show_register) = signal(false);
    let toast: RwSignal<Option<ToastMsg>> = RwSignal::new(None);

    // OTP state — phone stored as digits only
    let otp_phone = RwSignal::new(String::new()); // digits only
    let otp_sent = RwSignal::new(false);
    let otp_code = RwSignal::new(String::new());

    // Which login method is active: "phone", "email"
    let active_method = RwSignal::new("phone".to_string());

    let login_pending = login.pending();
    let register_pending = register.pending();
    let send_otp_pending = send_otp.pending();
    let verify_otp_pending = verify_otp.pending();

    let login_success = RwSignal::new(false);
    let register_success = RwSignal::new(false);
    let verify_success = RwSignal::new(false);

    let register_form: NodeRef<leptos::html::Form> = NodeRef::new();

    // Shared error toast helper
    let show_error = move |e: &ServerFnError| {
        let msg = clean_error(e);
        toast.set(None);
        leptos::task::spawn_local(async move {
            toast.set(Some(ToastMsg {
                message: msg,
                is_error: true,
            }));
        });
    };

    // Handle email login result
    Effect::new(move |_| match login.value().get() {
        Some(Ok(_)) => {
            login_success.set(true);
            #[cfg(feature = "hydrate")]
            let _ = js_sys::eval("setTimeout(() => { window.location.href = '/'; }, 700)");
        }
        Some(Err(e)) => show_error(&e),
        None => {}
    });

    // Handle register result
    Effect::new(move |_| match register.value().get() {
        Some(Ok(_)) => {
            if let Some(form) = register_form.get() {
                form.reset();
            }
            register_success.set(true);
            #[cfg(feature = "hydrate")]
            let _ = js_sys::eval("setTimeout(() => { window.location.href = '/'; }, 700)");
        }
        Some(Err(e)) => show_error(&e),
        None => {}
    });

    // Handle send OTP result
    Effect::new(move |_| match send_otp.value().get() {
        Some(Ok(_)) => {
            otp_sent.set(true);
            toast.set(Some(ToastMsg {
                message: "OTP sent!".to_string(),
                is_error: false,
            }));
        }
        Some(Err(e)) => show_error(&e),
        None => {}
    });

    // Handle verify OTP result
    Effect::new(move |_| match verify_otp.value().get() {
        Some(Ok(result)) => {
            verify_success.set(true);
            #[cfg(feature = "hydrate")]
            {
                let url = if result == crate::auth::OtpResult::NewAccount {
                    "/profile"
                } else {
                    "/"
                };
                let _ = js_sys::eval(&format!(
                    "setTimeout(() => {{ window.location.href = '{}'; }}, 700)",
                    url
                ));
            }
            let _ = result;
        }
        Some(Err(e)) => show_error(&e),
        None => {}
    });

    let on_send_otp = move |_| {
        let digits = otp_phone.get_untracked();
        if digits.is_empty() {
            toast.set(Some(ToastMsg {
                message: "Please enter your phone number".to_string(),
                is_error: true,
            }));
            return;
        }
        if digits.len() != 10 {
            toast.set(Some(ToastMsg {
                message: "Please enter a valid 10-digit phone number".to_string(),
                is_error: true,
            }));
            return;
        }
        let full_phone = format!("+91{}", digits);
        send_otp.dispatch(SendOtp { phone: full_phone });
    };

    let on_verify_otp = move |_| {
        let full_phone = format!("+91{}", otp_phone.get_untracked());
        let code = otp_code.get_untracked();
        verify_otp.dispatch(VerifyOtp {
            phone: full_phone,
            code: code.trim().to_string(),
        });
    };

    view! {
        <div class="login-page">
            <div class="login-card">
                // Phone OTP section (primary)
                <div class="login-methods">
                    <button
                        class="method-tab"
                        class:active=move || active_method.get() == "phone"
                        on:click=move |_| active_method.set("phone".to_string())
                    >"Phone"</button>
                    <button
                        class="method-tab"
                        class:active=move || active_method.get() == "email"
                        on:click=move |_| active_method.set("email".to_string())
                    >"Email"</button>
                </div>

                <Show when=move || active_method.get() == "phone">
                    <div class="auth-form">
                        <Show when=move || !otp_sent.get()>
                            <div class="phone-input-row">
                                <span class="country-code-label">"+91"</span>
                                <input
                                    type="tel"
                                    inputmode="numeric"
                                    class="phone-number-input"
                                    placeholder="98765 43210"
                                    maxlength="10"
                                    prop:value=move || otp_phone.get()
                                    on:input=move |ev| {
                                        let mut val = digits_only(&event_target_value(&ev));
                                        val.truncate(10);
                                        otp_phone.set(val);
                                    }
                                />
                            </div>
                            <button
                                class="auth-submit"
                                disabled=move || send_otp_pending.get()
                                on:click=on_send_otp
                            >
                                <Show when=move || send_otp_pending.get()>
                                    <span class="auth-spinner"></span>
                                </Show>
                                {move || if send_otp_pending.get() { "Sending..." } else { "Send OTP" }}
                            </button>
                        </Show>
                        <p class="otp-voice-note">"You'll receive a voice call with the OTP (SMS unavailable due to regulations)."</p>
                        <Show when=move || otp_sent.get()>
                            <p class="otp-hint">"Enter the 6-digit code sent to "<strong>{move || format!("+91 {}", otp_phone.get())}</strong></p>
                            <input
                                type="text"
                                inputmode="numeric"
                                maxlength="6"
                                placeholder="000000"
                                class="otp-input"
                                prop:value=move || otp_code.get()
                                on:input=move |ev| {
                                    let val = digits_only(&event_target_value(&ev));
                                    otp_code.set(val);
                                }
                            />
                            <button
                                class="auth-submit"
                                class:auth-submit--success=move || verify_success.get()
                                disabled=move || verify_otp_pending.get() || verify_success.get()
                                on:click=on_verify_otp
                            >
                                <Show when=move || verify_otp_pending.get()>
                                    <span class="auth-spinner"></span>
                                </Show>
                                {move || if verify_success.get() {
                                    "✓ Signed in!"
                                } else if verify_otp_pending.get() {
                                    "Verifying..."
                                } else {
                                    "Verify & Sign In"
                                }}
                            </button>
                            <button
                                class="auth-link otp-resend"
                                on:click=move |_| {
                                    otp_sent.set(false);
                                    otp_code.set(String::new());
                                    send_otp.value().set(None);
                                }
                            >"Change number"</button>
                        </Show>
                    </div>
                </Show>

                <Show when=move || active_method.get() == "email">
                    <Show when=move || !show_register.get()>
                        <div class="auth-form">
                            <ActionForm action=login>
                                <input type="email" name="email" placeholder="Email" required />
                                <input type="password" name="password" placeholder="Password" required />
                                <button
                                    type="submit"
                                    class="auth-submit"
                                    class:auth-submit--success=move || login_success.get()
                                    disabled=move || login_pending.get() || login_success.get()
                                >
                                    <Show when=move || login_pending.get()>
                                        <span class="auth-spinner"></span>
                                    </Show>
                                    {move || if login_success.get() {
                                        "✓ Signed in!"
                                    } else if login_pending.get() {
                                        "Signing in..."
                                    } else {
                                        "Sign in"
                                    }}
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
                                    class:auth-submit--success=move || register_success.get()
                                    disabled=move || register_pending.get() || register_success.get()
                                >
                                    <Show when=move || register_pending.get()>
                                        <span class="auth-spinner"></span>
                                    </Show>
                                    {move || if register_success.get() {
                                        "✓ Account created!"
                                    } else if register_pending.get() {
                                        "Creating account..."
                                    } else {
                                        "Create account"
                                    }}
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
                </Show>

                <div class="auth-divider"><span>"or"</span></div>

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
