#[cfg(feature = "ssr")]
use crate::auth::UserRole;
use crate::db::{Wod, WodMovement, WodSection};
use leptos::prelude::*;

// ---- WOD CRUD ----

#[server]
pub async fn list_wods() -> Result<Vec<Wod>, ServerFnError> {
    let pool = crate::db::db().await?;
    crate::db::list_wods_db(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn list_wods_for_date(date: String) -> Result<Vec<Wod>, ServerFnError> {
    let pool = crate::db::db().await?;
    crate::db::list_wods_for_date_db(&pool, &date)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn create_wod(
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
pub async fn update_wod(
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
pub async fn delete_wod(id: String) -> Result<(), ServerFnError> {
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
pub async fn list_exercises_for_wod() -> Result<Vec<(String, String)>, ServerFnError> {
    let pool = crate::db::db().await?;
    crate::db::list_exercises_db(&pool)
        .await
        .map(|exs| exs.into_iter().map(|e| (e.id, e.name)).collect())
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ---- WOD Sections ----

#[server]
pub async fn list_wod_sections(wod_id: String) -> Result<Vec<WodSection>, ServerFnError> {
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = wod_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::list_wod_sections_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn create_wod_section(
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
pub async fn update_wod_section(
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
pub async fn delete_wod_section(id: String) -> Result<(), ServerFnError> {
    crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::delete_wod_section_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ---- Section Movements ----

#[server]
pub async fn get_section_movements(section_id: String) -> Result<Vec<WodMovement>, ServerFnError> {
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = section_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::get_wod_movements_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn add_section_movement(
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
pub async fn update_section_movement(
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
pub async fn delete_section_movement(id: String) -> Result<(), ServerFnError> {
    crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::delete_wod_movement_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
