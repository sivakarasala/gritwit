#[cfg(feature = "ssr")]
use crate::db::SectionScoreInput;
use crate::db::{Wod, WodMovement, WodSection};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
fn validate_date(date: &str) -> Result<(), ServerFnError> {
    if date.is_empty() {
        return Err(ServerFnError::new("Date is required"));
    }
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| ServerFnError::new("Invalid date format"))?;
    Ok(())
}

/// Load a WOD with its sections and movements for scoring.
#[server]
pub async fn get_wod_for_scoring(
    wod_id: String,
) -> Result<(Wod, Vec<WodSection>, Vec<WodMovement>), ServerFnError> {
    let _user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = wod_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let (wod, sections) = crate::db::get_wod_with_sections_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let movements = crate::db::get_all_wod_movements_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok((wod, sections, movements))
}

/// Look up which WOD a section belongs to.
#[server]
pub async fn get_wod_by_section(
    section_id: String,
) -> Result<(Wod, Vec<WodSection>, Vec<WodMovement>), ServerFnError> {
    let _user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let sec_uuid: uuid::Uuid = section_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;

    let (wod_id,): (uuid::Uuid,) = sqlx::query_as("SELECT wod_id FROM wod_sections WHERE id = $1")
        .bind(sec_uuid)
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let (wod, sections) = crate::db::get_wod_with_sections_db(&pool, wod_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let movements = crate::db::get_all_wod_movements_db(&pool, wod_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok((wod, sections, movements))
}

/// Get today's WODs for the WOD picker.
#[server]
pub async fn get_todays_wods() -> Result<Vec<Wod>, ServerFnError> {
    let _user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let today = chrono::Local::now().date_naive().to_string();
    let wods = crate::db::list_wods_for_date_db(&pool, &today)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(wods)
}

/// Submit WOD scores.
#[server]
pub async fn submit_wod_scores(
    wod_id: String,
    workout_date: String,
    notes: String,
    scores_json: String,
) -> Result<String, ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    validate_date(&workout_date)?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let wod_uuid: uuid::Uuid = wod_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;

    if let Some(_existing_id) =
        crate::db::has_wod_score_db(&pool, user_uuid, wod_uuid, &workout_date)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        return Err(ServerFnError::new(format!(
            "You've already logged this WOD on {}",
            workout_date
        )));
    }

    let sections: Vec<(SectionScoreInput, String)> = serde_json::from_str(&scores_json)
        .map_err(|e| ServerFnError::new(format!("Invalid scores data: {}", e)))?;

    let notes_opt = if notes.is_empty() {
        None
    } else {
        Some(notes.as_str())
    };

    let log_id = crate::db::submit_wod_score_db(
        &pool,
        user_uuid,
        wod_uuid,
        &workout_date,
        notes_opt,
        &sections,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(log_id.to_string())
}

/// Submit a custom (non-WOD) workout log with exercises.
#[server]
pub async fn submit_custom_workout(
    workout_date: String,
    notes: String,
    exercises_json: String,
) -> Result<String, ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    validate_date(&workout_date)?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;

    let notes_opt = if notes.is_empty() {
        None
    } else {
        Some(notes.as_str())
    };

    let exercises: Vec<crate::db::ExerciseSetInput> = serde_json::from_str(&exercises_json)
        .map_err(|_| ServerFnError::new("Invalid exercise data"))?;

    if exercises.is_empty() {
        return Err(ServerFnError::new("Add at least one exercise"));
    }

    let log_id =
        crate::db::submit_custom_workout_db(&pool, user_uuid, &workout_date, notes_opt, &exercises)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(log_id.to_string())
}

/// Load an existing workout log with its exercises for editing.
#[server]
pub async fn get_workout_for_edit(
    log_id: String,
) -> Result<(crate::db::WorkoutLog, Vec<crate::db::WorkoutExercise>), ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let log_uuid: uuid::Uuid = log_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;

    let log: crate::db::WorkoutLog = sqlx::query_as(
        r#"SELECT id::text, workout_date::text, notes, is_rx, wod_id::text
           FROM workout_logs WHERE id = $1 AND user_id = $2"#,
    )
    .bind(log_uuid)
    .bind(user_uuid)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Workout not found: {}", e)))?;

    let exercises = crate::db::list_workout_exercises_db(&pool, log_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok((log, exercises))
}

/// Update an existing custom workout.
#[server]
pub async fn update_custom_workout(
    log_id: String,
    workout_date: String,
    notes: String,
    exercises_json: String,
) -> Result<(), ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    validate_date(&workout_date)?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let log_uuid: uuid::Uuid = log_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;

    let notes_opt = if notes.is_empty() {
        None
    } else {
        Some(notes.as_str())
    };

    let exercises: Vec<crate::db::ExerciseSetInput> = serde_json::from_str(&exercises_json)
        .map_err(|_| ServerFnError::new("Invalid exercise data"))?;

    if exercises.is_empty() {
        return Err(ServerFnError::new("Add at least one exercise"));
    }

    crate::db::update_custom_workout_db(
        &pool,
        log_uuid,
        user_uuid,
        &workout_date,
        notes_opt,
        &exercises,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

/// Delete a workout log.
#[server]
pub async fn delete_workout_log(log_id: String) -> Result<(), ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let log_uuid: uuid::Uuid = log_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;

    crate::db::delete_workout_log_db(&pool, log_uuid, user_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

/// Load existing WOD section scores for editing.
#[server]
pub async fn get_wod_scores_for_edit(
    log_id: String,
) -> Result<(String, Vec<crate::db::SectionLog>), ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let log_uuid: uuid::Uuid = log_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;

    let log: crate::db::WorkoutLog = sqlx::query_as(
        "SELECT id::text, workout_date::text, notes, is_rx, wod_id::text
         FROM workout_logs WHERE id = $1 AND user_id = $2",
    )
    .bind(log_uuid)
    .bind(user_uuid)
    .fetch_one(&pool)
    .await
    .map_err(|_| ServerFnError::new("Workout not found"))?;

    let scores = crate::db::get_section_logs_db(&pool, log_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok((log.notes.unwrap_or_default(), scores))
}

/// Update an existing WOD log's scores.
#[server]
pub async fn update_wod_scores(
    log_id: String,
    workout_date: String,
    notes: String,
    scores_json: String,
) -> Result<(), ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    validate_date(&workout_date)?;
    let pool = crate::db::db().await?;
    let log_uuid: uuid::Uuid = log_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;

    let notes_opt: Option<&str> = if notes.is_empty() { None } else { Some(&notes) };

    let sections: Vec<(SectionScoreInput, String)> =
        serde_json::from_str(&scores_json).map_err(|_| ServerFnError::new("Invalid score data"))?;

    sqlx::query(
        "UPDATE workout_logs SET notes = $1, workout_date = $2::date, updated_at = now()
         WHERE id = $3 AND user_id = $4",
    )
    .bind(notes_opt)
    .bind(&workout_date)
    .bind(log_uuid)
    .bind(user_uuid)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    sqlx::query("DELETE FROM section_logs WHERE workout_log_id = $1")
        .bind(log_uuid)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    for (score, _section_type) in &sections {
        let sec_uuid: uuid::Uuid = score
            .section_id
            .parse()
            .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
        let (section_log_id,): (uuid::Uuid,) = sqlx::query_as(
            r#"INSERT INTO section_logs
               (workout_log_id, section_id, finish_time_seconds, rounds_completed,
                extra_reps, weight_kg, notes, is_rx, skipped)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               RETURNING id"#,
        )
        .bind(log_uuid)
        .bind(sec_uuid)
        .bind(score.finish_time_seconds)
        .bind(score.rounds_completed)
        .bind(score.extra_reps)
        .bind(score.weight_kg)
        .bind(score.notes.as_deref())
        .bind(score.is_rx)
        .bind(score.skipped)
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        // Insert per-movement logs
        for ml in &score.movement_logs {
            let mov_uuid: uuid::Uuid = ml
                .movement_id
                .parse()
                .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
            sqlx::query(
                r#"INSERT INTO movement_logs (section_log_id, movement_id, reps, sets, weight_kg, notes)
                   VALUES ($1, $2, $3, $4, $5, $6)"#,
            )
            .bind(section_log_id)
            .bind(mov_uuid)
            .bind(ml.reps)
            .bind(ml.sets)
            .bind(ml.weight_kg)
            .bind(&ml.notes)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        }
    }

    Ok(())
}

/// Load existing movement logs for editing a workout.
#[server]
pub async fn get_movement_logs_for_edit(
    log_id: String,
) -> Result<Vec<crate::db::MovementLog>, ServerFnError> {
    let _user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let log_uuid: uuid::Uuid = log_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::get_movement_logs_for_workout_db(&pool, log_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Load movements for a specific section.
#[server]
pub async fn get_section_movements_for_log(
    section_id: String,
) -> Result<Vec<WodMovement>, ServerFnError> {
    let _user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = section_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::get_wod_movements_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Get exercises for the picker.
#[server]
pub async fn list_exercises_for_picker() -> Result<Vec<crate::db::Exercise>, ServerFnError> {
    let _user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let exercises = crate::db::list_exercises_db(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(exercises)
}
