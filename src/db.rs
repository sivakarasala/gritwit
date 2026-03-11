use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
pub async fn db() -> Result<sqlx::PgPool, leptos::prelude::ServerFnError> {
    leptos::prelude::use_context::<sqlx::PgPool>()
        .ok_or_else(|| leptos::prelude::ServerFnError::new("Database pool not found in context"))
}

// ---- Models ----

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Exercise {
    pub id: String,
    pub name: String,
    pub category: String,
    pub movement_type: Option<String>,
    pub muscle_groups: Vec<String>,
    pub description: Option<String>,
    pub demo_video_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct WorkoutLog {
    pub id: String,
    pub workout_date: String,
    pub workout_type: String,
    pub name: Option<String>,
    pub notes: Option<String>,
    pub duration_seconds: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct WorkoutWithExerciseName {
    pub id: String,
    pub exercise_name: String,
    pub sets: Option<i32>,
    pub reps: Option<i32>,
    pub weight_kg: Option<f32>,
    pub duration_seconds: Option<i32>,
    pub notes: Option<String>,
}

// ---- Exercise Queries ----

#[cfg(feature = "ssr")]
pub async fn list_exercises_db(pool: &sqlx::PgPool) -> Result<Vec<Exercise>, sqlx::Error> {
    sqlx::query_as::<_, Exercise>(
        r#"SELECT
            id::text, name, category,
            movement_type, muscle_groups, description,
            demo_video_url
        FROM exercises
        ORDER BY name"#,
    )
    .fetch_all(pool)
    .await
}

#[cfg(feature = "ssr")]
pub async fn create_exercise_db(
    pool: &sqlx::PgPool,
    name: &str,
    category: &str,
    movement_type: Option<&str>,
    muscle_groups: &[String],
    description: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO exercises (name, category, movement_type, muscle_groups, description)
        VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(name)
    .bind(category)
    .bind(movement_type)
    .bind(muscle_groups)
    .bind(description)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn delete_exercise_db(pool: &sqlx::PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM exercises WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ---- Workout Log Queries ----

#[cfg(feature = "ssr")]
pub async fn list_workout_logs_db(
    pool: &sqlx::PgPool,
    limit: i64,
) -> Result<Vec<WorkoutLog>, sqlx::Error> {
    sqlx::query_as::<_, WorkoutLog>(
        r#"SELECT
            id::text, workout_date::text, workout_type,
            name, notes, duration_seconds
        FROM workout_logs
        ORDER BY workout_date DESC, created_at DESC
        LIMIT $1"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

#[cfg(feature = "ssr")]
pub async fn create_workout_log_db(
    pool: &sqlx::PgPool,
    workout_date: &str,
    workout_type: &str,
    name: Option<&str>,
    notes: Option<&str>,
    duration_seconds: Option<i32>,
) -> Result<uuid::Uuid, sqlx::Error> {
    let date: chrono::NaiveDate = workout_date
        .parse()
        .map_err(|e| sqlx::Error::Protocol(format!("Invalid date: {}", e)))?;
    let row: (uuid::Uuid,) = sqlx::query_as(
        r#"INSERT INTO workout_logs (workout_date, workout_type, name, notes, duration_seconds)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id"#,
    )
    .bind(date)
    .bind(workout_type)
    .bind(name)
    .bind(notes)
    .bind(duration_seconds)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

#[cfg(feature = "ssr")]
pub async fn add_workout_exercise_db(
    pool: &sqlx::PgPool,
    workout_log_id: uuid::Uuid,
    exercise_id: uuid::Uuid,
    sets: Option<i32>,
    reps: Option<i32>,
    weight_kg: Option<f32>,
    duration_seconds: Option<i32>,
    sort_order: i32,
    notes: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO workout_exercises (workout_log_id, exercise_id, sets, reps, weight_kg, duration_seconds, sort_order, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
    )
    .bind(workout_log_id)
    .bind(exercise_id)
    .bind(sets)
    .bind(reps)
    .bind(weight_kg)
    .bind(duration_seconds)
    .bind(sort_order)
    .bind(notes)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn get_workout_exercises_db(
    pool: &sqlx::PgPool,
    workout_log_id: uuid::Uuid,
) -> Result<Vec<WorkoutWithExerciseName>, sqlx::Error> {
    sqlx::query_as::<_, WorkoutWithExerciseName>(
        r#"SELECT
            we.id::text, e.name as exercise_name,
            we.sets, we.reps, we.weight_kg,
            we.duration_seconds, we.notes
        FROM workout_exercises we
        JOIN exercises e ON e.id = we.exercise_id
        WHERE we.workout_log_id = $1
        ORDER BY we.sort_order"#,
    )
    .bind(workout_log_id)
    .fetch_all(pool)
    .await
}

// ---- Stats Queries ----

#[cfg(feature = "ssr")]
pub async fn count_exercises_db(pool: &sqlx::PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM exercises")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

#[cfg(feature = "ssr")]
pub async fn count_workouts_db(pool: &sqlx::PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM workout_logs")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}
