use crate::auth::{get_me, AuthUser, UserRole};
use crate::pages::{
    AdminPage, ExercisesPage, HistoryPage, HomePage, LogWorkoutPage, LoginPage, ProfilePage,
    WodPage,
};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes, A},
    StaticSegment,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover"/>
                <link rel="icon" type="image/x-icon" href="/favicon.ico"/>
                <link rel="manifest" href="/manifest.json"/>
                <meta name="theme-color" content="#0f0f1a"/>
                <script>"(function(){var t=localStorage.getItem('theme')||'dark';document.documentElement.setAttribute('data-theme',t)})()"</script>
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

    view! {
        <Stylesheet id="leptos" href="/pkg/gritwit.css"/>
        <Title text="GrindIt"/>

        <Router>
            <Suspense fallback=|| view! { <div class="login-page"><p>"Loading..."</p></div> }>
                {move || {
                    user.get().map(|result| {
                        match result {
                            Ok(Some(auth_user)) => {
                                view! { <AuthenticatedApp user=auth_user/> }.into_any()
                            }
                            Ok(None) => {
                                view! { <LoginPage/> }.into_any()
                            }
                            Err(e) => {
                                leptos::logging::log!("get_me error: {}", e);
                                view! { <LoginPage/> }.into_any()
                            }
                        }
                    })
                }}
            </Suspense>
        </Router>
    }
}

#[component]
fn AuthenticatedApp(user: AuthUser) -> impl IntoView {
    let is_admin = user.role == UserRole::Admin;
    provide_context(user.clone());

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
                <a href="/profile" class="top-bar__avatar">
                    {if let Some(ref url) = user.avatar_url {
                        view! { <img src={url.clone()} class="top-bar__avatar-img" referrerpolicy="no-referrer"/> }.into_any()
                    } else {
                        view! { <span class="top-bar__avatar-initials">{user.initials()}</span> }.into_any()
                    }}
                </a>
            </div>
        </header>
        <main>
            <Routes fallback=|| "Page not found.".into_view()>
                <Route path=StaticSegment("") view=HomePage/>
                <Route path=StaticSegment("exercises") view=ExercisesPage/>
                <Route path=StaticSegment("wod") view=WodPage/>
                <Route path=StaticSegment("log") view=LogWorkoutPage/>
                <Route path=StaticSegment("history") view=HistoryPage/>
                <Route path=StaticSegment("profile") view=ProfilePage/>
                <Route path=StaticSegment("admin") view=AdminPage/>
            </Routes>
        </main>
        <BottomNav is_admin=is_admin/>
    }
}

#[component]
fn BottomNav(is_admin: bool) -> impl IntoView {
    view! {
        <nav class="bottom-nav">
            <A href="/" attr:class="tab-item" exact=true>
                <span class="tab-icon tab-icon--home"></span>
                <span class="tab-label">"Home"</span>
            </A>
            <A href="/exercises" attr:class="tab-item">
                <span class="tab-icon tab-icon--exercises"></span>
                <span class="tab-label">"Exercises"</span>
            </A>
            <A href="/wod" attr:class="tab-item">
                <span class="tab-icon tab-icon--wod"></span>
                <span class="tab-label">"WOD"</span>
            </A>
            <A href="/log" attr:class="tab-item">
                <span class="tab-icon tab-icon--plus"></span>
                <span class="tab-label">"Log"</span>
            </A>
            <A href="/history" attr:class="tab-item">
                <span class="tab-icon tab-icon--history"></span>
                <span class="tab-label">"History"</span>
            </A>
            {is_admin.then(|| view! {
                <A href="/admin" attr:class="tab-item">
                    <span class="tab-icon tab-icon--admin"></span>
                    <span class="tab-label">"Admin"</span>
                </A>
            })}
        </nav>
    }
}
