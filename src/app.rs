use crate::auth::{get_me, AuthUser, UserRole};
use crate::pages::{
    AdminPage, ExercisesPage, HistoryPage, HomePage, LogWorkoutPage, LoginPage, ProfilePage,
    WodPage,
};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover"/>
                <link rel="icon" type="image/png" href="/favicon.png"/>
                <link rel="apple-touch-icon" href="/apple-touch-icon.png"/>
                <link rel="manifest" href="/manifest.json"/>
                <meta name="theme-color" content="#0f0f1a"/>
                <script>"(function(){var t=localStorage.getItem('theme')||'dark';document.documentElement.setAttribute('data-theme',t)})()"</script>
                <script>"window.__pwaInstallPrompt=null;window.addEventListener('beforeinstallprompt',function(e){e.preventDefault();window.__pwaInstallPrompt=e});window.__isIos=/iPad|iPhone|iPod/.test(navigator.userAgent)&&!window.MSStream;window.__isStandalone=window.matchMedia('(display-mode:standalone)').matches||navigator.standalone===true"</script>
                <link rel="preconnect" href="https://fonts.googleapis.com"/>
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous"/>
                <link href="https://fonts.googleapis.com/css2?family=Russo+One&display=swap" rel="stylesheet"/>
                <meta name="apple-mobile-web-app-capable" content="yes"/>
                <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent"/>
                <meta name="apple-mobile-web-app-title" content="GrindIt"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let user = Resource::new(|| (), |_| get_me());
    let is_admin = RwSignal::new(false);

    // Sync admin status outside the Suspense render to avoid reactive scope issues
    Effect::new(move |_| {
        let admin = user
            .get()
            .and_then(|r| r.ok())
            .flatten()
            .map(|u| u.role == UserRole::Admin)
            .unwrap_or(false);
        is_admin.set(admin);
    });

    view! {
        <Stylesheet id="leptos" href="/pkg/gritwit.css"/>
        <Title text="GrindIt"/>

        <Router>
            <ScrollReset/>
            <Transition fallback=|| view! { <div class="login-page"><p>"Loading..."</p></div> }>
                {move || {
                    user.get().map(|result| {
                        let auth_user = match result {
                            Ok(Some(u)) => Some(u),
                            _ => None,
                        };
                        if let Some(ref u) = auth_user {
                            provide_context(u.clone());
                        }
                        let is_authed = auth_user.is_some();

                        view! {
                            <Header user=auth_user/>
                            <main>
                                <Routes fallback=|| "Page not found.".into_view()>
                                    // Public pages
                                    <Route path=StaticSegment("exercises") view=ExercisesPage/>
                                    <Route path=StaticSegment("wod") view=WodPage/>
                                    <Route path=StaticSegment("login") view=LoginPage/>

                                    // Auth-gated pages — show LoginPage if anonymous
                                    <Route path=StaticSegment("") view=move || {
                                        if is_authed { view! { <HomePage/> }.into_any() }
                                        else { view! { <LoginPage/> }.into_any() }
                                    }/>
                                    <Route path=StaticSegment("log") view=move || {
                                        if is_authed { view! { <LogWorkoutPage/> }.into_any() }
                                        else { view! { <LoginPage/> }.into_any() }
                                    }/>
                                    <Route path=StaticSegment("history") view=move || {
                                        if is_authed { view! { <HistoryPage/> }.into_any() }
                                        else { view! { <LoginPage/> }.into_any() }
                                    }/>
                                    <Route path=StaticSegment("profile") view=move || {
                                        if is_authed { view! { <ProfilePage/> }.into_any() }
                                        else { view! { <LoginPage/> }.into_any() }
                                    }/>
                                    <Route path=StaticSegment("admin") view=move || {
                                        if is_authed { view! { <AdminPage/> }.into_any() }
                                        else { view! { <LoginPage/> }.into_any() }
                                    }/>
                                </Routes>
                            </main>
                        }
                        .into_any()
                    })
                }}
            </Transition>
            <InstallBanner/>
            <BottomNav is_admin=is_admin/>
        </Router>
    }
}

#[component]
fn Header(user: Option<AuthUser>) -> impl IntoView {
    view! {
        <header class="top-bar">
            <span class="top-bar__logo">"Grind"<span class="top-bar__flame"></span>"t"</span>
            <div class="top-bar__actions">
                <button class="theme-toggle" on:click=move |_| {
                    #[cfg(feature = "hydrate")]
                    { crate::voice::toggle_theme(); }
                }>
                    <span class="theme-icon theme-icon--sun"></span>
                    <span class="theme-icon theme-icon--moon"></span>
                </button>
                {user.map(|u| view! {
                    <a href="/profile" class="top-bar__avatar">
                        {if let Some(ref url) = u.avatar_url {
                            view! { <img src={url.clone()} class="top-bar__avatar-img" referrerpolicy="no-referrer"/> }.into_any()
                        } else {
                            view! { <span class="top-bar__avatar-initials">{u.initials()}</span> }.into_any()
                        }}
                    </a>
                })}
            </div>
        </header>
    }
}

#[component]
fn InstallBanner() -> impl IntoView {
    let show = RwSignal::new(false);
    let is_ios = RwSignal::new(false);

    // Defer detection to after hydration so SSR and initial client render agree (show=false).
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;
            let global = js_sys::global();

            let standalone = js_sys::Reflect::get(&global, &JsValue::from_str("__isStandalone"))
                .unwrap_or(JsValue::FALSE)
                .as_bool()
                .unwrap_or(false);

            let dismissed = {
                let ls = js_sys::Reflect::get(&global, &JsValue::from_str("localStorage"))
                    .unwrap_or(JsValue::UNDEFINED);
                if !ls.is_undefined() {
                    let get_fn = js_sys::Reflect::get(&ls, &JsValue::from_str("getItem"))
                        .unwrap_or(JsValue::UNDEFINED);
                    if let Ok(f) = get_fn.dyn_into::<js_sys::Function>() {
                        f.call1(&ls, &JsValue::from_str("pwa_install_dismissed"))
                            .unwrap_or(JsValue::NULL)
                            == JsValue::from_str("1")
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            if !standalone && !dismissed {
                let ios = js_sys::Reflect::get(&global, &JsValue::from_str("__isIos"))
                    .unwrap_or(JsValue::FALSE)
                    .as_bool()
                    .unwrap_or(false);
                let has_prompt =
                    !js_sys::Reflect::get(&global, &JsValue::from_str("__pwaInstallPrompt"))
                        .unwrap_or(JsValue::NULL)
                        .is_null();

                if ios || has_prompt {
                    show.set(true);
                    is_ios.set(ios);
                }
            }
        }
    });

    let dismiss = move |_| {
        show.set(false);
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;
            let global = js_sys::global();
            let ls = js_sys::Reflect::get(&global, &JsValue::from_str("localStorage"))
                .unwrap_or(JsValue::UNDEFINED);
            if !ls.is_undefined() {
                if let Ok(f) = js_sys::Reflect::get(&ls, &JsValue::from_str("setItem"))
                    .unwrap_or(JsValue::UNDEFINED)
                    .dyn_into::<js_sys::Function>()
                {
                    let _ = f.call2(
                        &ls,
                        &JsValue::from_str("pwa_install_dismissed"),
                        &JsValue::from_str("1"),
                    );
                }
            }
        }
    };

    let install = move |_| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;
            let global = js_sys::global();
            let prompt = js_sys::Reflect::get(&global, &JsValue::from_str("__pwaInstallPrompt"))
                .unwrap_or(JsValue::NULL);
            if !prompt.is_null() {
                if let Ok(f) = js_sys::Reflect::get(&prompt, &JsValue::from_str("prompt"))
                    .unwrap_or(JsValue::UNDEFINED)
                    .dyn_into::<js_sys::Function>()
                {
                    let _ = f.call0(&prompt);
                }
            }
            show.set(false);
        }
    };

    move || {
        if !show.get() {
            return ().into_any();
        }
        if is_ios.get() {
            view! {
                <div class="install-banner">
                    <div class="install-banner__text">
                        <strong>"Install GrindIt"</strong>
                        <span class="install-banner__sub">
                            "Tap "
                            <svg class="install-banner__share-icon" viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
                                <path d="M16 5l-1.42 1.42-1.59-1.59V16h-1.98V4.83L9.42 6.42 8 5l4-4 4 4zm4 5v11a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V10c0-1.1.9-2 2-2h3v2H6v11h12V10h-3V8h3a2 2 0 0 1 2 2z"/>
                            </svg>
                            " then \"Add to Home Screen\""
                        </span>
                    </div>
                    <button class="install-banner__close" on:click=dismiss>"×"</button>
                </div>
            }
            .into_any()
        } else {
            view! {
                <div class="install-banner">
                    <div class="install-banner__text">
                        <strong>"Install GrindIt"</strong>
                        <span class="install-banner__sub">"Add to your home screen for the best experience"</span>
                    </div>
                    <button class="install-banner__btn" on:click=install>"Install"</button>
                    <button class="install-banner__close" on:click=dismiss>"×"</button>
                </div>
            }
            .into_any()
        }
    }
}

/// Reset scroll and dismiss keyboard on route change (fixes iOS viewport bugs).
#[component]
fn ScrollReset() -> impl IntoView {
    let pathname = leptos_router::hooks::use_location().pathname;
    Effect::new(move |_| {
        let _ = pathname.get();
        #[cfg(feature = "hydrate")]
        {
            // Blur active element + scroll main to top instantly via JS
            let _ = js_sys::eval(
                "if(document.activeElement)document.activeElement.blur();\
                 var m=document.querySelector('main');if(m){m.scrollTo({top:0,behavior:'instant'})}",
            );
        }
    });
}

#[component]
fn BottomNav(is_admin: RwSignal<bool>) -> impl IntoView {
    let pathname = leptos_router::hooks::use_location().pathname;

    view! {
        <nav class="bottom-nav">
            <a href="/" class="tab-item" class:active=move || pathname.get() == "/">
                <span class="tab-icon tab-icon--home"></span>
                <span class="tab-label">"Home"</span>
            </a>
            <a href="/exercises" class="tab-item" class:active=move || pathname.get().starts_with("/exercises")>
                <span class="tab-icon tab-icon--exercises"></span>
                <span class="tab-label">"Exercises"</span>
            </a>
            <a href="/wod" class="tab-item" class:active=move || pathname.get().starts_with("/wod")>
                <span class="tab-icon tab-icon--wod"></span>
                <span class="tab-label">"WOD"</span>
            </a>
            <a href="/log" class="tab-item" class:active=move || pathname.get().starts_with("/log")>
                <span class="tab-icon tab-icon--plus"></span>
                <span class="tab-label">"Log"</span>
            </a>
            <a href="/history" class="tab-item" class:active=move || pathname.get().starts_with("/history")>
                <span class="tab-icon tab-icon--history"></span>
                <span class="tab-label">"History"</span>
            </a>
            {move || is_admin.get().then(|| view! {
                <a href="/admin" class="tab-item" class:active=move || pathname.get().starts_with("/admin")>
                    <span class="tab-icon tab-icon--admin"></span>
                    <span class="tab-label">"Admin"</span>
                </a>
            })}
        </nav>
    }
}
