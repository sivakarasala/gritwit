mod section_movements_panel;
pub mod week_calendar;
mod wod_card;
mod wod_form;
mod wod_section_card;
mod wod_sections_panel;

use crate::auth::{AuthUser, UserRole};
use crate::components::DeleteModal;
use crate::db::{Wod, WodMovement, WodSection};
use leptos::prelude::*;

use week_calendar::WeeklyCalendar;
use wod_card::WodCard;
use wod_form::WodForm;

// ---- Server functions: WOD CRUD ----

#[server]
async fn list_wods() -> Result<Vec<Wod>, ServerFnError> {
    let pool = crate::db::db().await?;
    crate::db::list_wods_db(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub(super) async fn list_wods_for_date(date: String) -> Result<Vec<Wod>, ServerFnError> {
    let pool = crate::db::db().await?;
    crate::db::list_wods_for_date_db(&pool, &date)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn create_wod(
    title: String,
    description: String,
    workout_type: String,
    time_cap_minutes: String,
    programmed_date: String,
) -> Result<String, ServerFnError> {
    let user = crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let time_cap = if time_cap_minutes.is_empty() {
        None
    } else {
        time_cap_minutes.parse::<i32>().ok()
    };
    let desc = if description.is_empty() {
        None
    } else {
        Some(description.as_str())
    };
    crate::db::create_wod_db(
        &pool,
        &title,
        desc,
        &workout_type,
        time_cap,
        &programmed_date,
        Some(user_uuid),
    )
    .await
    .map(|id| id.to_string())
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn update_wod(
    id: String,
    title: String,
    description: String,
    workout_type: String,
    time_cap_minutes: String,
    programmed_date: String,
) -> Result<(), ServerFnError> {
    crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let time_cap = if time_cap_minutes.is_empty() {
        None
    } else {
        time_cap_minutes.parse::<i32>().ok()
    };
    let desc = if description.is_empty() {
        None
    } else {
        Some(description.as_str())
    };
    crate::db::update_wod_db(
        &pool,
        uuid,
        &title,
        desc,
        &workout_type,
        time_cap,
        &programmed_date,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn delete_wod(id: String) -> Result<(), ServerFnError> {
    crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::delete_wod_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub(super) async fn list_exercises_for_wod() -> Result<Vec<(String, String)>, ServerFnError> {
    let pool = crate::db::db().await?;
    crate::db::list_exercises_db(&pool)
        .await
        .map(|exs| exs.into_iter().map(|e| (e.id, e.name)).collect())
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ---- Server functions: WOD Sections ----

#[server]
pub(super) async fn list_wod_sections(wod_id: String) -> Result<Vec<WodSection>, ServerFnError> {
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = wod_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::list_wod_sections_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub(super) async fn create_wod_section(
    wod_id: String,
    phase: String,
    title: String,
    section_type: String,
    time_cap_minutes: String,
    rounds: String,
    notes: String,
) -> Result<String, ServerFnError> {
    crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let wod_uuid: uuid::Uuid = wod_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let title_opt = if title.is_empty() {
        None
    } else {
        Some(title.as_str())
    };
    let notes_opt = if notes.is_empty() {
        None
    } else {
        Some(notes.as_str())
    };
    let time_cap_opt = if time_cap_minutes.is_empty() {
        None
    } else {
        time_cap_minutes.parse::<i32>().ok()
    };
    let rounds_opt = if rounds.is_empty() {
        None
    } else {
        rounds.parse::<i32>().ok()
    };
    crate::db::create_wod_section_db(
        &pool,
        wod_uuid,
        &phase,
        title_opt,
        &section_type,
        time_cap_opt,
        rounds_opt,
        notes_opt,
        0,
    )
    .await
    .map(|id| id.to_string())
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub(super) async fn update_wod_section(
    id: String,
    phase: String,
    title: String,
    section_type: String,
    time_cap_minutes: String,
    rounds: String,
    notes: String,
) -> Result<(), ServerFnError> {
    crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let title_opt = if title.is_empty() {
        None
    } else {
        Some(title.as_str())
    };
    let notes_opt = if notes.is_empty() {
        None
    } else {
        Some(notes.as_str())
    };
    let time_cap_opt = if time_cap_minutes.is_empty() {
        None
    } else {
        time_cap_minutes.parse::<i32>().ok()
    };
    let rounds_opt = if rounds.is_empty() {
        None
    } else {
        rounds.parse::<i32>().ok()
    };
    crate::db::update_wod_section_db(
        &pool,
        uuid,
        &phase,
        title_opt,
        &section_type,
        time_cap_opt,
        rounds_opt,
        notes_opt,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub(super) async fn delete_wod_section(id: String) -> Result<(), ServerFnError> {
    crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::delete_wod_section_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ---- Server functions: Section Movements ----

#[server]
pub(super) async fn get_section_movements(
    section_id: String,
) -> Result<Vec<WodMovement>, ServerFnError> {
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = section_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::get_wod_movements_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub(super) async fn add_section_movement(
    section_id: String,
    exercise_id: String,
    rep_scheme: String,
    weight_kg_male: String,
    weight_kg_female: String,
    notes: String,
) -> Result<(), ServerFnError> {
    crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let sec_uuid: uuid::Uuid = section_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let ex_uuid: uuid::Uuid = exercise_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let rep_opt = if rep_scheme.is_empty() {
        None
    } else {
        Some(rep_scheme.as_str())
    };
    let notes_opt = if notes.is_empty() {
        None
    } else {
        Some(notes.as_str())
    };
    let male_opt: Option<f32> = if weight_kg_male.is_empty() {
        None
    } else {
        weight_kg_male.parse().ok()
    };
    let female_opt: Option<f32> = if weight_kg_female.is_empty() {
        None
    } else {
        weight_kg_female.parse().ok()
    };
    crate::db::add_wod_movement_db(
        &pool, sec_uuid, ex_uuid, rep_opt, male_opt, female_opt, notes_opt, 0,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub(super) async fn update_section_movement(
    id: String,
    exercise_id: String,
    rep_scheme: String,
    weight_kg_male: String,
    weight_kg_female: String,
    notes: String,
) -> Result<(), ServerFnError> {
    crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let ex_uuid: uuid::Uuid = exercise_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let rep_opt = if rep_scheme.is_empty() {
        None
    } else {
        Some(rep_scheme.as_str())
    };
    let notes_opt = if notes.is_empty() {
        None
    } else {
        Some(notes.as_str())
    };
    let male_opt: Option<f32> = if weight_kg_male.is_empty() {
        None
    } else {
        weight_kg_male.parse().ok()
    };
    let female_opt: Option<f32> = if weight_kg_female.is_empty() {
        None
    } else {
        weight_kg_female.parse().ok()
    };
    crate::db::update_wod_movement_db(
        &pool, uuid, ex_uuid, rep_opt, male_opt, female_opt, notes_opt,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub(super) async fn delete_section_movement(id: String) -> Result<(), ServerFnError> {
    crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::delete_wod_movement_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ---- Helper functions (pub(super) so sub-modules can use them) ----

pub(super) fn phase_label(p: &str) -> &'static str {
    match p {
        "warmup" => "Warm-Up",
        "strength" => "Strength",
        "conditioning" => "Conditioning",
        "cooldown" => "Cool Down",
        "optional" => "Optional",
        "personal" => "Personal",
        _ => "Section",
    }
}

pub(super) fn section_type_label(t: &str) -> &'static str {
    match t {
        "fortime" => "For Time",
        "amrap" => "AMRAP",
        "emom" => "EMOM",
        "strength" => "Strength",
        _ => "",
    }
}

pub(super) fn phase_class(p: &str) -> &'static str {
    match p {
        "warmup" => "phase-badge--warmup",
        "strength" => "phase-badge--strength",
        "conditioning" => "phase-badge--conditioning",
        "cooldown" => "phase-badge--cooldown",
        "optional" => "phase-badge--optional",
        "personal" => "phase-badge--personal",
        _ => "",
    }
}

pub(super) fn wod_type_label(t: &str) -> &'static str {
    match t {
        "amrap" => "AMRAP",
        "fortime" => "FOR TIME",
        "emom" => "EMOM",
        "tabata" => "TABATA",
        "strength" => "STRENGTH",
        _ => "CUSTOM",
    }
}

pub(super) fn wod_type_class(t: &str) -> &'static str {
    match t {
        "amrap" => "wod-badge--amrap",
        "fortime" => "wod-badge--fortime",
        "emom" => "wod-badge--emom",
        "tabata" => "wod-badge--tabata",
        "strength" => "wod-badge--strength",
        _ => "wod-badge--custom",
    }
}

// ---- WodPage ----

#[component]
pub fn WodPage() -> impl IntoView {
    let is_coach = use_context::<AuthUser>()
        .map(|u| matches!(u.role, UserRole::Coach | UserRole::Admin))
        .unwrap_or(false);

    let create_action = ServerAction::<CreateWod>::new();
    let delete_action = ServerAction::<DeleteWod>::new();
    let update_action = ServerAction::<UpdateWod>::new();

    let selected_date: RwSignal<String> = RwSignal::new(week_calendar::today_iso());

    let wods = Resource::new(
        move || {
            (
                selected_date.get(),
                create_action.version().get(),
                delete_action.version().get(),
                update_action.version().get(),
            )
        },
        |(date, _, _, _)| list_wods_for_date(date),
    );

    let show_form = RwSignal::new(false);
    let title_input = RwSignal::new(String::new());
    let desc_input = RwSignal::new(String::new());
    let type_input = RwSignal::new("fortime".to_string());
    let cap_input = RwSignal::new(String::new());
    let date_input = RwSignal::new(String::new());

    let editing_wod: RwSignal<Option<String>> = RwSignal::new(None);

    let show_delete_wod = RwSignal::new(false);
    let pending_delete_wod_id = RwSignal::new(String::new());

    let fab_view = if is_coach {
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

    let form_view = move || {
        if is_coach && show_form.get() {
            view! {
                <WodForm
                    create_action=create_action
                    show_form=show_form
                    title_input=title_input
                    desc_input=desc_input
                    type_input=type_input
                    cap_input=cap_input
                    date_input=date_input
                />
            }
            .into_any()
        } else {
            ().into_view().into_any()
        }
    };

    let list_view = view! {
        <Suspense fallback=|| view! { <p class="loading">"Loading WODs..."</p> }>
            {move || wods.get().map(|result| match result {
                Err(e) => view! {
                    <p class="wod-error">{format!("Error: {}", e)}</p>
                }.into_any(),
                Ok(list) if list.is_empty() => view! {
                    <div class="empty-state">
                        <p class="empty-title">"No WOD for this day"</p>
                        {is_coach.then(|| view! {
                            <p class="empty-sub">"Use + to program a WOD"</p>
                        })}
                    </div>
                }.into_any(),
                Ok(list) => view! {
                    <div class="wod-list">
                        {list.into_iter().map(|wod| {
                            view! {
                                <WodCard
                                    wod=wod
                                    is_coach=is_coach
                                    editing_wod=editing_wod
                                    update_action=update_action
                                    pending_delete_wod_id=pending_delete_wod_id
                                    show_delete_wod=show_delete_wod
                                />
                            }
                        }).collect_view()}
                    </div>
                }.into_any(),
            })}
        </Suspense>
    }
    .into_any();

    view! {
        <div class="wod-page">
            {fab_view}
            {form_view}
            <div style=move || if show_form.get() { "display:none" } else { "" }>
                <WeeklyCalendar selected_date=selected_date />
                {list_view}
                <DeleteModal
                    show=show_delete_wod
                    title="Delete this WOD?"
                    subtitle="All sections and movements will also be removed. This cannot be undone."
                    confirm_label="Delete"
                    on_confirm=move || {
                        delete_action.dispatch(DeleteWod {
                            id: pending_delete_wod_id.get_untracked(),
                        });
                    }
                />
            </div>
        </div>
    }
}
