use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
static POOL: std::sync::OnceLock<sqlx::PgPool> = std::sync::OnceLock::new();

/// Call once from main() to make the pool globally available.
#[cfg(feature = "ssr")]
pub fn init_pool(pool: sqlx::PgPool) {
    POOL.set(pool).expect("Pool already initialized");
}

#[cfg(feature = "ssr")]
pub async fn db() -> Result<sqlx::PgPool, leptos::prelude::ServerFnError> {
    // Try Leptos context first (available inside leptos_routes_with_context),
    // fall back to the global pool for server function calls outside that scope.
    leptos::prelude::use_context::<sqlx::PgPool>()
        .or_else(|| POOL.get().cloned())
        .ok_or_else(|| leptos::prelude::ServerFnError::new("Database pool not initialized"))
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
    pub is_rx: bool,
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

// ---- User Queries ----

#[cfg(feature = "ssr")]
pub async fn get_user_by_id(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
) -> Result<crate::auth::AuthUser, sqlx::Error> {
    let row: (String, String, String, Option<String>, String) = sqlx::query_as(
        r#"SELECT id::text, email, display_name, avatar_url, role::text
           FROM users WHERE id = $1"#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    let role = match row.4.as_str() {
        "admin" => crate::auth::UserRole::Admin,
        "coach" => crate::auth::UserRole::Coach,
        _ => crate::auth::UserRole::Athlete,
    };

    Ok(crate::auth::AuthUser {
        id: row.0,
        email: row.1,
        display_name: row.2,
        avatar_url: row.3,
        role,
    })
}

#[cfg(feature = "ssr")]
pub async fn list_users_db(pool: &sqlx::PgPool) -> Result<Vec<crate::auth::AuthUser>, sqlx::Error> {
    let rows: Vec<(String, String, String, Option<String>, String)> = sqlx::query_as(
        r#"SELECT id::text, email, display_name, avatar_url, role::text
           FROM users ORDER BY created_at"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let role = match row.4.as_str() {
                "admin" => crate::auth::UserRole::Admin,
                "coach" => crate::auth::UserRole::Coach,
                _ => crate::auth::UserRole::Athlete,
            };
            crate::auth::AuthUser {
                id: row.0,
                email: row.1,
                display_name: row.2,
                avatar_url: row.3,
                role,
            }
        })
        .collect())
}

#[cfg(feature = "ssr")]
pub async fn update_user_role_db(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    new_role: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET role = $1::user_role, updated_at = now() WHERE id = $2")
        .bind(new_role)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
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
    created_by: Option<uuid::Uuid>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO exercises (name, category, movement_type, muscle_groups, description, created_by)
        VALUES ($1, $2, $3, $4, $5, $6)"#,
    )
    .bind(name)
    .bind(category)
    .bind(movement_type)
    .bind(muscle_groups)
    .bind(description)
    .bind(created_by)
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
    user_id: uuid::Uuid,
    limit: i64,
) -> Result<Vec<WorkoutLog>, sqlx::Error> {
    sqlx::query_as::<_, WorkoutLog>(
        r#"SELECT
            id::text, workout_date::text, workout_type,
            name, notes, duration_seconds, is_rx
        FROM workout_logs
        WHERE user_id = $1
        ORDER BY workout_date DESC, created_at DESC
        LIMIT $2"#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

#[cfg(feature = "ssr")]
pub async fn create_workout_log_db(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    workout_date: &str,
    workout_type: &str,
    name: Option<&str>,
    notes: Option<&str>,
    duration_seconds: Option<i32>,
    is_rx: bool,
) -> Result<uuid::Uuid, sqlx::Error> {
    let date: chrono::NaiveDate = workout_date
        .parse()
        .map_err(|e| sqlx::Error::Protocol(format!("Invalid date: {}", e)))?;
    let row: (uuid::Uuid,) = sqlx::query_as(
        r#"INSERT INTO workout_logs (user_id, workout_date, workout_type, name, notes, duration_seconds, is_rx)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id"#,
    )
    .bind(user_id)
    .bind(date)
    .bind(workout_type)
    .bind(name)
    .bind(notes)
    .bind(duration_seconds)
    .bind(is_rx)
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
pub async fn count_workouts_db(pool: &sqlx::PgPool, user_id: uuid::Uuid) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM workout_logs WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// Count consecutive days (ending today or yesterday) that the user has logged workouts.
#[cfg(feature = "ssr")]
pub async fn streak_days_db(pool: &sqlx::PgPool, user_id: uuid::Uuid) -> Result<i64, sqlx::Error> {
    // Get distinct workout dates in descending order, starting from today
    let dates: Vec<(chrono::NaiveDate,)> = sqlx::query_as(
        r#"SELECT DISTINCT workout_date
           FROM workout_logs
           WHERE user_id = $1 AND workout_date <= CURRENT_DATE
           ORDER BY workout_date DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    if dates.is_empty() {
        return Ok(0);
    }

    let today = chrono::Local::now().date_naive();
    let mut streak = 0i64;
    let mut expected = today;

    for (date,) in dates {
        // Allow starting from today or yesterday
        if streak == 0 && date == today - chrono::Duration::days(1) {
            expected = today - chrono::Duration::days(1);
        }
        if date == expected {
            streak += 1;
            expected -= chrono::Duration::days(1);
        } else if date < expected {
            break;
        }
    }

    Ok(streak)
}

/// Leaderboard entry for a given date range.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LeaderboardEntry {
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub workout_count: i64,
}

#[cfg(feature = "ssr")]
pub async fn leaderboard_db(pool: &sqlx::PgPool, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    // Simple leaderboard: users ranked by total workouts logged this week
    let rows: Vec<(String, Option<String>, i64)> = sqlx::query_as(
        r#"SELECT u.display_name, u.avatar_url, COUNT(wl.id) as workout_count
           FROM users u
           LEFT JOIN workout_logs wl ON wl.user_id = u.id
               AND wl.workout_date >= date_trunc('week', CURRENT_DATE)::date
           GROUP BY u.id, u.display_name, u.avatar_url
           HAVING COUNT(wl.id) > 0
           ORDER BY workout_count DESC
           LIMIT $1"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(display_name, avatar_url, workout_count)| {
        LeaderboardEntry { display_name, avatar_url, workout_count }
    }).collect())
}

/// Get workouts for a specific date, scoped to user.
#[cfg(feature = "ssr")]
pub async fn list_workouts_by_date_db(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    date: chrono::NaiveDate,
) -> Result<Vec<WorkoutLog>, sqlx::Error> {
    sqlx::query_as::<_, WorkoutLog>(
        r#"SELECT
            id::text, workout_date::text, workout_type,
            name, notes, duration_seconds, is_rx
        FROM workout_logs
        WHERE user_id = $1 AND workout_date = $2
        ORDER BY created_at DESC"#,
    )
    .bind(user_id)
    .bind(date)
    .fetch_all(pool)
    .await
}
