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
    pub notes: Option<String>,
    pub is_rx: bool,
    pub wod_id: Option<String>,
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
#[allow(clippy::too_many_arguments)]
pub async fn create_exercise_db(
    pool: &sqlx::PgPool,
    name: &str,
    category: &str,
    movement_type: Option<&str>,
    muscle_groups: &[String],
    description: Option<&str>,
    demo_video_url: Option<&str>,
    created_by: Option<uuid::Uuid>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO exercises (name, category, movement_type, muscle_groups, description, demo_video_url, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
    )
    .bind(name)
    .bind(category)
    .bind(movement_type)
    .bind(muscle_groups)
    .bind(description)
    .bind(demo_video_url)
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

#[cfg(feature = "ssr")]
#[allow(clippy::too_many_arguments)]
pub async fn update_exercise_db(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
    name: &str,
    category: &str,
    movement_type: Option<&str>,
    description: Option<&str>,
    demo_video_url: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE exercises
           SET name = $2, category = $3, movement_type = $4,
               description = $5, demo_video_url = $6
           WHERE id = $1"#,
    )
    .bind(id)
    .bind(name)
    .bind(category)
    .bind(movement_type)
    .bind(description)
    .bind(demo_video_url)
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
        r#"SELECT id::text, workout_date::text, notes, is_rx, wod_id::text
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
    wod_id: Option<uuid::Uuid>,
    workout_date: &str,
    notes: Option<&str>,
    is_rx: bool,
) -> Result<uuid::Uuid, sqlx::Error> {
    let date: chrono::NaiveDate = workout_date
        .parse()
        .map_err(|e| sqlx::Error::Protocol(format!("Invalid date: {}", e)))?;
    let row: (uuid::Uuid,) = sqlx::query_as(
        r#"INSERT INTO workout_logs (user_id, wod_id, workout_date, notes, is_rx)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id"#,
    )
    .bind(user_id)
    .bind(wod_id)
    .bind(date)
    .bind(notes)
    .bind(is_rx)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
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
pub async fn count_workouts_db(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
) -> Result<i64, sqlx::Error> {
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
pub async fn leaderboard_db(
    pool: &sqlx::PgPool,
    limit: i64,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
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

    Ok(rows
        .into_iter()
        .map(
            |(display_name, avatar_url, workout_count)| LeaderboardEntry {
                display_name,
                avatar_url,
                workout_count,
            },
        )
        .collect())
}

/// Get workouts for a specific date, scoped to user.
#[cfg(feature = "ssr")]
pub async fn list_workouts_by_date_db(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    date: chrono::NaiveDate,
) -> Result<Vec<WorkoutLog>, sqlx::Error> {
    sqlx::query_as::<_, WorkoutLog>(
        r#"SELECT id::text, workout_date::text, notes, is_rx, wod_id::text
        FROM workout_logs
        WHERE user_id = $1 AND workout_date = $2
        ORDER BY created_at DESC"#,
    )
    .bind(user_id)
    .bind(date)
    .fetch_all(pool)
    .await
}

// ---- WOD Models ----

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Wod {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub workout_type: String,
    pub time_cap_minutes: Option<i32>,
    pub programmed_date: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct WodSection {
    pub id: String,
    pub wod_id: String,
    pub phase: String,
    pub title: Option<String>,
    pub section_type: String,
    pub time_cap_minutes: Option<i32>,
    pub rounds: Option<i32>,
    pub notes: Option<String>,
    pub sort_order: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct WodMovement {
    pub id: String,
    pub section_id: String,
    pub exercise_id: String,
    pub exercise_name: String,
    pub rep_scheme: Option<String>,
    pub weight_kg_male: Option<f32>,
    pub weight_kg_female: Option<f32>,
    pub notes: Option<String>,
    pub sort_order: i32,
}

// ---- WOD Queries ----

#[cfg(feature = "ssr")]
pub async fn list_wods_db(pool: &sqlx::PgPool) -> Result<Vec<Wod>, sqlx::Error> {
    sqlx::query_as::<_, Wod>(
        r#"SELECT id::text, title, description, workout_type,
                  time_cap_minutes, programmed_date::text
           FROM wods
           ORDER BY programmed_date DESC, created_at DESC"#,
    )
    .fetch_all(pool)
    .await
}

#[cfg(feature = "ssr")]
pub async fn list_wods_for_date_db(
    pool: &sqlx::PgPool,
    date: &str,
) -> Result<Vec<Wod>, sqlx::Error> {
    sqlx::query_as::<_, Wod>(
        r#"SELECT id::text, title, description, workout_type,
                  time_cap_minutes, programmed_date::text
           FROM wods
           WHERE programmed_date = $1::date
           ORDER BY created_at ASC"#,
    )
    .bind(date)
    .fetch_all(pool)
    .await
}

#[cfg(feature = "ssr")]
pub async fn create_wod_db(
    pool: &sqlx::PgPool,
    title: &str,
    description: Option<&str>,
    workout_type: &str,
    time_cap_minutes: Option<i32>,
    programmed_date: &str,
    created_by: Option<uuid::Uuid>,
) -> Result<uuid::Uuid, sqlx::Error> {
    let date: chrono::NaiveDate = programmed_date
        .parse()
        .map_err(|e| sqlx::Error::Protocol(format!("Invalid date: {}", e)))?;
    let row: (uuid::Uuid,) = sqlx::query_as(
        r#"INSERT INTO wods (title, description, workout_type, time_cap_minutes, programmed_date, created_by)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id"#,
    )
    .bind(title)
    .bind(description)
    .bind(workout_type)
    .bind(time_cap_minutes)
    .bind(date)
    .bind(created_by)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

#[cfg(feature = "ssr")]
pub async fn delete_wod_db(pool: &sqlx::PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM wods WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn delete_wod_movement_db(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM wod_movements WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn get_wod_movements_db(
    pool: &sqlx::PgPool,
    section_id: uuid::Uuid,
) -> Result<Vec<WodMovement>, sqlx::Error> {
    sqlx::query_as::<_, WodMovement>(
        r#"SELECT wm.id::text, wm.section_id::text, wm.exercise_id::text, e.name as exercise_name,
                  wm.rep_scheme, wm.weight_kg_male, wm.weight_kg_female, wm.notes, wm.sort_order
           FROM wod_movements wm
           JOIN exercises e ON e.id = wm.exercise_id
           WHERE wm.section_id = $1
           ORDER BY wm.sort_order, wm.created_at"#,
    )
    .bind(section_id)
    .fetch_all(pool)
    .await
}

#[cfg(feature = "ssr")]
#[allow(clippy::too_many_arguments)]
pub async fn add_wod_movement_db(
    pool: &sqlx::PgPool,
    section_id: uuid::Uuid,
    exercise_id: uuid::Uuid,
    rep_scheme: Option<&str>,
    weight_kg_male: Option<f32>,
    weight_kg_female: Option<f32>,
    notes: Option<&str>,
    sort_order: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO wod_movements (section_id, exercise_id, rep_scheme, weight_kg_male, weight_kg_female, notes, sort_order)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
    )
    .bind(section_id)
    .bind(exercise_id)
    .bind(rep_scheme)
    .bind(weight_kg_male)
    .bind(weight_kg_female)
    .bind(notes)
    .bind(sort_order)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(feature = "ssr")]
#[allow(clippy::too_many_arguments)]
pub async fn update_wod_db(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
    title: &str,
    description: Option<&str>,
    workout_type: &str,
    time_cap_minutes: Option<i32>,
    programmed_date: &str,
) -> Result<(), sqlx::Error> {
    let date: chrono::NaiveDate = programmed_date
        .parse()
        .map_err(|e| sqlx::Error::Protocol(format!("Invalid date: {}", e)))?;
    sqlx::query(
        r#"UPDATE wods
           SET title = $2, description = $3, workout_type = $4,
               time_cap_minutes = $5, programmed_date = $6
           WHERE id = $1"#,
    )
    .bind(id)
    .bind(title)
    .bind(description)
    .bind(workout_type)
    .bind(time_cap_minutes)
    .bind(date)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(feature = "ssr")]
#[allow(clippy::too_many_arguments)]
pub async fn update_wod_movement_db(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
    exercise_id: uuid::Uuid,
    rep_scheme: Option<&str>,
    weight_kg_male: Option<f32>,
    weight_kg_female: Option<f32>,
    notes: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE wod_movements
           SET exercise_id = $2, rep_scheme = $3, weight_kg_male = $4, weight_kg_female = $5, notes = $6
           WHERE id = $1"#,
    )
    .bind(id)
    .bind(exercise_id)
    .bind(rep_scheme)
    .bind(weight_kg_male)
    .bind(weight_kg_female)
    .bind(notes)
    .execute(pool)
    .await?;
    Ok(())
}

// ---- WOD Section Queries ----

#[cfg(feature = "ssr")]
pub async fn list_wod_sections_db(
    pool: &sqlx::PgPool,
    wod_id: uuid::Uuid,
) -> Result<Vec<WodSection>, sqlx::Error> {
    sqlx::query_as::<_, WodSection>(
        r#"SELECT id::text, wod_id::text, phase::text, title, section_type::text,
                  time_cap_minutes, rounds, notes, sort_order
           FROM wod_sections
           WHERE wod_id = $1
           ORDER BY sort_order, created_at"#,
    )
    .bind(wod_id)
    .fetch_all(pool)
    .await
}

#[cfg(feature = "ssr")]
#[allow(clippy::too_many_arguments)]
pub async fn create_wod_section_db(
    pool: &sqlx::PgPool,
    wod_id: uuid::Uuid,
    phase: &str,
    title: Option<&str>,
    section_type: &str,
    time_cap_minutes: Option<i32>,
    rounds: Option<i32>,
    notes: Option<&str>,
    sort_order: i32,
) -> Result<uuid::Uuid, sqlx::Error> {
    let row: (uuid::Uuid,) = sqlx::query_as(
        r#"INSERT INTO wod_sections (wod_id, phase, title, section_type, time_cap_minutes, rounds, notes, sort_order)
           VALUES ($1, $2::wod_phase, $3, $4::section_type, $5, $6, $7, $8)
           RETURNING id"#,
    )
    .bind(wod_id)
    .bind(phase)
    .bind(title)
    .bind(section_type)
    .bind(time_cap_minutes)
    .bind(rounds)
    .bind(notes)
    .bind(sort_order)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

#[cfg(feature = "ssr")]
#[allow(clippy::too_many_arguments)]
pub async fn update_wod_section_db(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
    phase: &str,
    title: Option<&str>,
    section_type: &str,
    time_cap_minutes: Option<i32>,
    rounds: Option<i32>,
    notes: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE wod_sections
           SET phase = $2::wod_phase, title = $3, section_type = $4::section_type,
               time_cap_minutes = $5, rounds = $6, notes = $7
           WHERE id = $1"#,
    )
    .bind(id)
    .bind(phase)
    .bind(title)
    .bind(section_type)
    .bind(time_cap_minutes)
    .bind(rounds)
    .bind(notes)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn delete_wod_section_db(pool: &sqlx::PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM wod_sections WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
