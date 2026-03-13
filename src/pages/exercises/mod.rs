mod exercise_card;
mod exercise_form;

use crate::components::{DeleteModal, MultiSelect, SelectOption};
use crate::db::Exercise;
use exercise_card::ExerciseCard;
use exercise_form::ExerciseForm;
use leptos::prelude::*;

#[server]
async fn list_exercises() -> Result<Vec<Exercise>, ServerFnError> {
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
    crate::db::create_exercise_db(
        &pool,
        &name,
        &category,
        mt,
        &[],
        desc,
        video,
        Some(user_uuid),
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn update_exercise(
    id: String,
    name: String,
    category: String,
    movement_type: String,
    description: String,
    demo_video_url: String,
) -> Result<(), ServerFnError> {
    let _user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = id
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
    crate::db::update_exercise_db(&pool, uuid, &name, &category, mt, desc, video)
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
pub(crate) fn to_embed_url(url: &str) -> Option<String> {
    if url.contains("youtube.com/watch") {
        if let Some(pos) = url.find("v=") {
            let id = &url[pos + 2..];
            let id = id.split('&').next().unwrap_or(id);
            return Some(format!(
                "https://www.youtube.com/embed/{}?playsinline=1&enablejsapi=1",
                id
            ));
        }
    }
    if url.contains("youtu.be/") {
        if let Some(pos) = url.find("youtu.be/") {
            let id = &url[pos + 9..];
            let id = id.split('?').next().unwrap_or(id);
            return Some(format!(
                "https://www.youtube.com/embed/{}?playsinline=1&enablejsapi=1",
                id
            ));
        }
    }
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

/// Single source of truth for exercise categories.
/// Each entry: (value, label, badge, css_class)
pub(crate) const CATEGORIES: &[(&str, &str, &str, &str)] = &[
    ("conditioning", "Conditioning", "CON", "badge--conditioning"),
    ("gymnastics", "Gymnastics", "GYM", "badge--gymnastics"),
    (
        "weightlifting",
        "Weightlifting",
        "WL",
        "badge--weightlifting",
    ),
    ("powerlifting", "Powerlifting", "PWR", "badge--powerlifting"),
    ("cardio", "Cardio", "CRD", "badge--cardio"),
    ("bodybuilding", "Bodybuilding", "BB", "badge--bodybuilding"),
    ("strongman", "Strongman", "STR", "badge--strongman"),
    ("plyometrics", "Plyometrics", "PLY", "badge--plyometrics"),
    ("calisthenics", "Calisthenics", "CAL", "badge--calisthenics"),
    ("mobility", "Mobility", "MOB", "badge--mobility"),
    ("yoga", "Yoga", "YGA", "badge--yoga"),
    ("meditation", "Meditation", "MED", "badge--meditation"),
    ("breathing", "Breathing", "BRE", "badge--breathing"),
    ("chanting", "Chanting", "CHN", "badge--chanting"),
    ("sports", "Sports", "SPT", "badge--sports"),
    ("warmup", "Warm Up", "WRM", "badge--warmup"),
    ("cooldown", "Cool Down", "CLD", "badge--cooldown"),
];

pub(crate) fn category_badge(cat: &str) -> &'static str {
    CATEGORIES
        .iter()
        .find(|(v, _, _, _)| *v == cat)
        .map(|(_, _, b, _)| *b)
        .unwrap_or("GEN")
}

pub(crate) fn category_class(cat: &str) -> &'static str {
    CATEGORIES
        .iter()
        .find(|(v, _, _, _)| *v == cat)
        .map(|(_, _, _, c)| *c)
        .unwrap_or("")
}

pub(crate) fn category_select_options() -> Vec<SelectOption> {
    CATEGORIES
        .iter()
        .map(|(val, label, _, _)| SelectOption {
            value: val.to_string(),
            label: label.to_string(),
        })
        .collect()
}

#[component]
pub fn ExercisesPage() -> impl IntoView {
    use crate::auth::AuthUser;
    let is_authed = use_context::<AuthUser>().is_some();

    let create_action = ServerAction::<CreateExercise>::new();
    let delete_action = ServerAction::<DeleteExercise>::new();
    let update_action = ServerAction::<UpdateExercise>::new();

    let exercises = Resource::new(
        move || {
            (
                create_action.version().get(),
                delete_action.version().get(),
                update_action.version().get(),
            )
        },
        |_| list_exercises(),
    );

    let active_filters: RwSignal<Vec<String>> = RwSignal::new(vec![]);
    let show_form = RwSignal::new(false);
    let expanded_video = RwSignal::new(Option::<String>::None);
    let editing_exercise: RwSignal<Option<String>> = RwSignal::new(None);
    let show_delete = RwSignal::new(false);
    let pending_delete_id = RwSignal::new(String::new());

    let fab_view = if is_authed {
        view! {
            <button
                class={move || if show_form.get() { "fab fab--active" } else { "fab" }}
                on:click=move |_| show_form.update(|v| *v = !*v)
            >
                <span class="fab-icon"></span>
            </button>
        }
        .into_any()
    } else {
        ().into_view().into_any()
    };

    let filter_options: Vec<SelectOption> = CATEGORIES
        .iter()
        .map(|(v, l, _, _)| SelectOption {
            value: v.to_string(),
            label: l.to_string(),
        })
        .collect();

    view! {
        <div class="exercises-page">
            {fab_view}
            {move || {
                if show_form.get() {
                    view! { <ExerciseForm create_action=create_action show_form=show_form/> }.into_any()
                } else {
                    ().into_view().into_any()
                }
            }}
            <div style=move || if show_form.get() { "display:none" } else { "" }>
                <MultiSelect options=filter_options selected=active_filters placeholder="All Categories"/>
                <Suspense fallback=|| view! { <p class="loading">"Loading movements..."</p> }>
                    {move || {
                        let filters = active_filters.get();
                        exercises.get().map(|result| {
                            match result {
                                Ok(list) if list.is_empty() => {
                                    view! {
                                        <div class="empty-state">
                                            <p class="empty-title">"No movements yet"</p>
                                            <p class="empty-sub">"Tap + to build your library"</p>
                                        </div>
                                    }.into_any()
                                }
                                Ok(list) => {
                                    let filtered: Vec<Exercise> = if filters.is_empty() {
                                        list
                                    } else {
                                        list.into_iter().filter(|e| filters.contains(&e.category)).collect()
                                    };
                                    view! {
                                        <div class="exercise-grid">
                                            {filtered.into_iter().map(|ex| {
                                                view! {
                                                    <ExerciseCard
                                                        exercise=ex
                                                        expanded_video=expanded_video
                                                        editing_exercise=editing_exercise
                                                        update_action=update_action
                                                        pending_delete_id=pending_delete_id
                                                        show_delete=show_delete
                                                        is_authed=is_authed
                                                    />
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
            </div>
            <DeleteModal
                show=show_delete
                title="Delete this movement?"
                subtitle="This cannot be undone."
                confirm_label="Delete"
                on_confirm=move || {
                    delete_action.dispatch(DeleteExercise {
                        id: pending_delete_id.get_untracked(),
                    });
                }
            />
        </div>
    }
}
