use crate::pages::{ExercisesPage, HistoryPage, HomePage, LogWorkoutPage};
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
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="icon" type="image/x-icon" href="/favicon.ico"/>
                <link rel="manifest" href="/manifest.json"/>
                <meta name="theme-color" content="#1a1a2e"/>
                <meta name="apple-mobile-web-app-capable" content="yes"/>
                <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent"/>
                <meta name="apple-mobile-web-app-title" content="GritWit"/>
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

    view! {
        <Stylesheet id="leptos" href="/pkg/gritwit.css"/>
        <Title text="GritWit"/>

        <Router>
            <Header/>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=StaticSegment("exercises") view=ExercisesPage/>
                    <Route path=StaticSegment("log") view=LogWorkoutPage/>
                    <Route path=StaticSegment("history") view=HistoryPage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn Header() -> impl IntoView {
    view! {
        <header>
            <nav>
                <A href="/" attr:class="logo">"GritWit"</A>
                <div class="nav-links">
                    <A href="/exercises" attr:class="nav-link">"Exercises"</A>
                    <A href="/log" attr:class="nav-link">"Log"</A>
                    <A href="/history" attr:class="nav-link">"History"</A>
                </div>
            </nav>
        </header>
    }
}
