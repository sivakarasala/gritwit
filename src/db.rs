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
    pub scoring_type: String,
    pub created_by: Option<String>,
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
#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
    email: Option<String>,
    phone: Option<String>,
    display_name: String,
    avatar_url: Option<String>,
    role: String,
    gender: Option<String>,
}

#[cfg(feature = "ssr")]
impl From<UserRow> for crate::auth::AuthUser {
    fn from(row: UserRow) -> Self {
        let role = match row.role.as_str() {
            "admin" => crate::auth::UserRole::Admin,
            "coach" => crate::auth::UserRole::Coach,
            _ => crate::auth::UserRole::Athlete,
        };
        crate::auth::AuthUser {
            id: row.id,
            email: row.email,
            phone: row.phone,
            display_name: row.display_name,
            avatar_url: row.avatar_url,
            role,
            gender: row.gender,
        }
    }
}

#[cfg(feature = "ssr")]
pub async fn get_user_by_id(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
) -> Result<crate::auth::AuthUser, sqlx::Error> {
    let row: UserRow = sqlx::query_as(
        r#"SELECT id::text, email, phone, display_name, avatar_url, role::text, gender::text
           FROM users WHERE id = $1"#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(row.into())
}

#[cfg(feature = "ssr")]
pub async fn list_users_db(pool: &sqlx::PgPool) -> Result<Vec<crate::auth::AuthUser>, sqlx::Error> {
    let rows: Vec<UserRow> = sqlx::query_as(
        r#"SELECT id::text, email, phone, display_name, avatar_url, role::text, gender::text
           FROM users ORDER BY created_at"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
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

#[cfg(feature = "ssr")]
pub async fn update_user_profile_db(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    display_name: &str,
    email: Option<&str>,
    phone: Option<&str>,
    gender: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE users
           SET display_name = $1,
               email = $2,
               phone = $3,
               gender = $4::gender,
               updated_at = now()
           WHERE id = $5"#,
    )
    .bind(display_name)
    .bind(email)
    .bind(phone)
    .bind(gender)
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
            demo_video_url, scoring_type, created_by::text
        FROM exercises
        WHERE deleted_at IS NULL
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
    scoring_type: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO exercises (name, category, movement_type, muscle_groups, description, demo_video_url, created_by, scoring_type)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
    )
    .bind(name)
    .bind(category)
    .bind(movement_type)
    .bind(muscle_groups)
    .bind(description)
    .bind(demo_video_url)
    .bind(created_by)
    .bind(scoring_type)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn delete_exercise_db(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
    requesting_user_id: uuid::Uuid,
    is_admin: bool,
) -> Result<(), sqlx::Error> {
    if is_admin {
        sqlx::query("UPDATE exercises SET deleted_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
    } else {
        let result = sqlx::query(
            "UPDATE exercises SET deleted_at = NOW() WHERE id = $1 AND created_by = $2 AND deleted_at IS NULL"
        )
        .bind(id)
        .bind(requesting_user_id)
        .execute(pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
    }
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
    scoring_type: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE exercises
           SET name = $2, category = $3, movement_type = $4,
               description = $5, demo_video_url = $6, scoring_type = $7
           WHERE id = $1"#,
    )
    .bind(id)
    .bind(name)
    .bind(category)
    .bind(movement_type)
    .bind(description)
    .bind(demo_video_url)
    .bind(scoring_type)
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

// ---- Section Log Models ----

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct SectionLog {
    pub id: String,
    pub workout_log_id: String,
    pub section_id: String,
    pub finish_time_seconds: Option<i32>,
    pub rounds_completed: Option<i32>,
    pub extra_reps: Option<i32>,
    pub weight_kg: Option<f32>,
    pub notes: Option<String>,
    pub is_rx: bool,
    pub skipped: bool,
    pub score_value: Option<i32>,
}

/// Section score enriched with section metadata, for history display.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct SectionScoreWithMeta {
    pub section_log_id: String,
    pub section_type: String,
    pub section_title: Option<String>,
    pub finish_time_seconds: Option<i32>,
    pub rounds_completed: Option<i32>,
    pub extra_reps: Option<i32>,
    pub weight_kg: Option<f32>,
    pub is_rx: bool,
    pub skipped: bool,
}

/// A movement log enriched with the exercise name.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct MovementLogWithName {
    pub id: String,
    pub section_log_id: String,
    pub exercise_name: String,
    pub scoring_type: String,
    pub reps: Option<i32>,
    pub sets: Option<i32>,
    pub weight_kg: Option<f32>,
    pub notes: Option<String>,
}

/// Input for submitting a single section score.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectionScoreInput {
    pub section_id: String,
    pub finish_time_seconds: Option<i32>,
    pub rounds_completed: Option<i32>,
    pub extra_reps: Option<i32>,
    pub weight_kg: Option<f32>,
    pub notes: Option<String>,
    pub is_rx: bool,
    pub skipped: bool,
    #[serde(default)]
    pub movement_logs: Vec<MovementLogInput>,
}

/// Input for logging results per movement within a section.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MovementLogInput {
    pub movement_id: String,
    pub reps: Option<i32>,
    pub sets: Option<i32>,
    pub weight_kg: Option<f32>,
    pub notes: Option<String>,
    #[serde(default)]
    pub set_details: Vec<MovementLogSetInput>,
}

/// A saved per-movement result (read from DB).
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MovementLog {
    pub id: String,
    pub section_log_id: String,
    pub movement_id: String,
    pub reps: Option<i32>,
    pub sets: Option<i32>,
    pub weight_kg: Option<f32>,
    pub notes: Option<String>,
}

/// Per-set data for a movement log (e.g., set 1: 9 reps @ 60kg, set 2: 8 reps @ 60kg).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct MovementLogSet {
    pub id: String,
    pub movement_log_id: String,
    pub set_number: i32,
    pub reps: Option<i32>,
    pub weight_kg: Option<f32>,
    pub notes: Option<String>,
    pub distance_meters: Option<f32>,
    pub calories: Option<i32>,
}

/// Input for inserting a single set row.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MovementLogSetInput {
    pub set_number: i32,
    pub reps: Option<i32>,
    pub weight_kg: Option<f32>,
    pub distance_meters: Option<f32>,
    pub calories: Option<i32>,
}

/// Leaderboard entry for a specific section.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectionLeaderboardEntry {
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub score_value: i32,
    pub is_rx: bool,
    pub finish_time_seconds: Option<i32>,
    pub rounds_completed: Option<i32>,
    pub extra_reps: Option<i32>,
    pub weight_kg: Option<f32>,
}

/// Personal best for a section.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonalBest {
    pub score_value: i32,
    pub is_rx: bool,
    pub logged_at: String,
}

#[cfg(feature = "ssr")]
pub async fn leaderboard_db(
    pool: &sqlx::PgPool,
    limit: i64,
    viewer_email: &str,
    is_viewer_admin: bool,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    // Test accounts are hidden from the leaderboard unless the viewer is
    // an admin or is the test user themselves.
    let test_emails: &[&str] = &["test@coach.com", "test@athlete.com"];
    let rows: Vec<(String, Option<String>, i64)> = sqlx::query_as(
        r#"SELECT u.display_name, u.avatar_url, COUNT(wl.id) as workout_count
           FROM users u
           LEFT JOIN workout_logs wl ON wl.user_id = u.id
               AND wl.workout_date >= date_trunc('week', CURRENT_DATE)::date
           WHERE ($2 OR u.email = $3 OR u.email != ALL($4))
           GROUP BY u.id, u.display_name, u.avatar_url
           HAVING COUNT(wl.id) > 0
           ORDER BY workout_count DESC
           LIMIT $1"#,
    )
    .bind(limit)
    .bind(is_viewer_admin)
    .bind(viewer_email)
    .bind(test_emails)
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

#[cfg(feature = "ssr")]
pub async fn list_workouts_by_date_range_db(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
) -> Result<Vec<WorkoutLog>, sqlx::Error> {
    sqlx::query_as::<_, WorkoutLog>(
        r#"SELECT id::text, workout_date::text, notes, is_rx, wod_id::text
        FROM workout_logs
        WHERE user_id = $1 AND workout_date >= $2 AND workout_date <= $3
        ORDER BY workout_date ASC, created_at DESC"#,
    )
    .bind(user_id)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
}

// ---- WOD Models ----

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Wod {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub workout_type: String,
    pub time_cap_minutes: Option<i32>,
    pub programmed_date: String,
    pub created_by: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    pub scoring_type: String,
}

// ---- WOD Queries ----

#[cfg(feature = "ssr")]
pub async fn list_wods_db(pool: &sqlx::PgPool) -> Result<Vec<Wod>, sqlx::Error> {
    sqlx::query_as::<_, Wod>(
        r#"SELECT id::text, title, description, workout_type,
                  time_cap_minutes, programmed_date::text, created_by::text
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
                  time_cap_minutes, programmed_date::text, created_by::text
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
                  wm.rep_scheme, wm.weight_kg_male, wm.weight_kg_female, wm.notes, wm.sort_order,
                  e.scoring_type
           FROM wod_movements wm
           JOIN exercises e ON e.id = wm.exercise_id
           WHERE wm.section_id = $1
           ORDER BY wm.sort_order, wm.created_at"#,
    )
    .bind(section_id)
    .fetch_all(pool)
    .await
}

/// Load all movements for all sections of a WOD in one query.
#[cfg(feature = "ssr")]
pub async fn get_all_wod_movements_db(
    pool: &sqlx::PgPool,
    wod_id: uuid::Uuid,
) -> Result<Vec<WodMovement>, sqlx::Error> {
    sqlx::query_as::<_, WodMovement>(
        r#"SELECT wm.id::text, wm.section_id::text, wm.exercise_id::text, e.name as exercise_name,
                  wm.rep_scheme, wm.weight_kg_male, wm.weight_kg_female, wm.notes, wm.sort_order,
                  e.scoring_type
           FROM wod_movements wm
           JOIN exercises e ON e.id = wm.exercise_id
           JOIN wod_sections ws ON ws.id = wm.section_id
           WHERE ws.wod_id = $1
           ORDER BY ws.sort_order, wm.sort_order, wm.created_at"#,
    )
    .bind(wod_id)
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

// ---- Section Log / Scoring Queries ----

/// Compute the score_value from section score inputs and section type.
/// - fortime: finish_time_seconds (lower = better)
/// - amrap/emom: rounds * 1000 + extra_reps (higher = better)
/// - strength: (weight_kg * 100) as i32 (higher = better)
/// - static/other: None
pub fn compute_score_value(
    section_type: &str,
    finish_time_seconds: Option<i32>,
    rounds_completed: Option<i32>,
    extra_reps: Option<i32>,
    weight_kg: Option<f32>,
) -> Option<i32> {
    match section_type {
        "fortime" => finish_time_seconds,
        "amrap" | "emom" => {
            let r = rounds_completed.unwrap_or(0);
            let e = extra_reps.unwrap_or(0);
            Some(r * 1000 + e)
        }
        "strength" => weight_kg.map(|w| (w * 100.0) as i32),
        _ => None,
    }
}

/// Submit scores for a WOD: creates a workout_log + section_logs in one transaction.
#[cfg(feature = "ssr")]
pub async fn submit_wod_score_db(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    wod_id: uuid::Uuid,
    workout_date: &str,
    notes: Option<&str>,
    sections: &[(SectionScoreInput, String)], // (input, section_type)
) -> Result<uuid::Uuid, sqlx::Error> {
    let date: chrono::NaiveDate = workout_date
        .parse()
        .map_err(|e| sqlx::Error::Protocol(format!("Invalid date: {}", e)))?;

    // Determine overall is_rx: true only if ALL non-skipped sections are RX
    let overall_rx = sections
        .iter()
        .filter(|(s, _)| !s.skipped)
        .all(|(s, _)| s.is_rx);

    let mut tx = pool.begin().await?;

    let (log_id,): (uuid::Uuid,) = sqlx::query_as(
        r#"INSERT INTO workout_logs (user_id, wod_id, workout_date, notes, is_rx)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id"#,
    )
    .bind(user_id)
    .bind(wod_id)
    .bind(date)
    .bind(notes)
    .bind(overall_rx)
    .fetch_one(&mut *tx)
    .await?;

    insert_sections_tx(&mut tx, log_id, sections).await?;
    tx.commit().await?;
    Ok(log_id)
}

#[cfg(feature = "ssr")]
async fn insert_sections_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    log_id: uuid::Uuid,
    sections: &[(SectionScoreInput, String)],
) -> Result<(), sqlx::Error> {
    for (input, section_type) in sections {
        let section_id: uuid::Uuid = input
            .section_id
            .parse()
            .map_err(|e| sqlx::Error::Protocol(format!("Invalid section_id: {}", e)))?;

        let score_value = if input.skipped {
            None
        } else {
            compute_score_value(
                section_type,
                input.finish_time_seconds,
                input.rounds_completed,
                input.extra_reps,
                input.weight_kg,
            )
        };

        let (section_log_id,): (uuid::Uuid,) = sqlx::query_as(
            r#"INSERT INTO section_logs
               (workout_log_id, section_id, finish_time_seconds, rounds_completed,
                extra_reps, weight_kg, notes, is_rx, skipped, score_value)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               RETURNING id"#,
        )
        .bind(log_id)
        .bind(section_id)
        .bind(input.finish_time_seconds)
        .bind(input.rounds_completed)
        .bind(input.extra_reps)
        .bind(input.weight_kg)
        .bind(&input.notes)
        .bind(input.is_rx)
        .bind(input.skipped)
        .bind(score_value)
        .fetch_one(&mut **tx)
        .await?;

        for ml in &input.movement_logs {
            let mov_id: uuid::Uuid = ml
                .movement_id
                .parse()
                .map_err(|e| sqlx::Error::Protocol(format!("Invalid movement_id: {}", e)))?;
            let (ml_id,): (uuid::Uuid,) = sqlx::query_as(
                r#"INSERT INTO movement_logs (section_log_id, movement_id, reps, sets, weight_kg, notes)
                   VALUES ($1, $2, $3, $4, $5, $6)
                   RETURNING id"#,
            )
            .bind(section_log_id)
            .bind(mov_id)
            .bind(ml.reps)
            .bind(ml.sets)
            .bind(ml.weight_kg)
            .bind(&ml.notes)
            .fetch_one(&mut **tx)
            .await?;

            for sd in &ml.set_details {
                sqlx::query(
                    r#"INSERT INTO movement_log_sets (movement_log_id, set_number, reps, weight_kg, distance_meters, calories)
                       VALUES ($1, $2, $3, $4, $5, $6)"#,
                )
                .bind(ml_id)
                .bind(sd.set_number)
                .bind(sd.reps)
                .bind(sd.weight_kg)
                .bind(sd.distance_meters)
                .bind(sd.calories)
                .execute(&mut **tx)
                .await?;
            }
        }
    }
    Ok(())
}

/// Add section scores to an existing workout log (used when logging individual sections).
#[cfg(feature = "ssr")]
pub async fn add_section_scores_db(
    pool: &sqlx::PgPool,
    log_id: uuid::Uuid,
    sections: &[(SectionScoreInput, String)],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    insert_sections_tx(&mut tx, log_id, sections).await?;
    tx.commit().await?;
    Ok(())
}

/// Get section logs for a workout log.
#[cfg(feature = "ssr")]
pub async fn get_section_logs_db(
    pool: &sqlx::PgPool,
    workout_log_id: uuid::Uuid,
) -> Result<Vec<SectionLog>, sqlx::Error> {
    sqlx::query_as::<_, SectionLog>(
        r#"SELECT id::text, workout_log_id::text, section_id::text,
                  finish_time_seconds, rounds_completed, extra_reps,
                  weight_kg, notes, is_rx, skipped, score_value
           FROM section_logs
           WHERE workout_log_id = $1
           ORDER BY created_at"#,
    )
    .bind(workout_log_id)
    .fetch_all(pool)
    .await
}

/// Get movement logs for a section log.
#[cfg(feature = "ssr")]
pub async fn get_movement_logs_db(
    pool: &sqlx::PgPool,
    section_log_id: uuid::Uuid,
) -> Result<Vec<MovementLog>, sqlx::Error> {
    sqlx::query_as::<_, MovementLog>(
        r#"SELECT id::text, section_log_id::text, movement_id::text,
                  reps, sets, weight_kg, notes
           FROM movement_logs
           WHERE section_log_id = $1
           ORDER BY created_at"#,
    )
    .bind(section_log_id)
    .fetch_all(pool)
    .await
}

/// Get all movement logs for a workout log (across all sections).
#[cfg(feature = "ssr")]
pub async fn get_movement_logs_for_workout_db(
    pool: &sqlx::PgPool,
    workout_log_id: uuid::Uuid,
) -> Result<Vec<MovementLog>, sqlx::Error> {
    sqlx::query_as::<_, MovementLog>(
        r#"SELECT ml.id::text, ml.section_log_id::text, ml.movement_id::text,
                  ml.reps, ml.sets, ml.weight_kg, ml.notes
           FROM movement_logs ml
           JOIN section_logs sl ON sl.id = ml.section_log_id
           WHERE sl.workout_log_id = $1
           ORDER BY ml.created_at"#,
    )
    .bind(workout_log_id)
    .fetch_all(pool)
    .await
}

/// Load section scores with section metadata for history display.
#[cfg(feature = "ssr")]
pub async fn get_section_scores_with_meta_db(
    pool: &sqlx::PgPool,
    workout_log_id: uuid::Uuid,
) -> Result<Vec<SectionScoreWithMeta>, sqlx::Error> {
    sqlx::query_as::<_, SectionScoreWithMeta>(
        r#"SELECT sl.id::text as section_log_id,
                  ws.section_type::text, ws.title as section_title,
                  sl.finish_time_seconds, sl.rounds_completed, sl.extra_reps,
                  sl.weight_kg, sl.is_rx, sl.skipped
           FROM section_logs sl
           JOIN wod_sections ws ON ws.id = sl.section_id
           WHERE sl.workout_log_id = $1
           ORDER BY ws.sort_order"#,
    )
    .bind(workout_log_id)
    .fetch_all(pool)
    .await
}

/// Get movement logs with exercise names for a workout.
#[cfg(feature = "ssr")]
pub async fn get_movement_logs_with_names_db(
    pool: &sqlx::PgPool,
    workout_log_id: uuid::Uuid,
) -> Result<Vec<MovementLogWithName>, sqlx::Error> {
    sqlx::query_as::<_, MovementLogWithName>(
        r#"SELECT ml.id::text, ml.section_log_id::text, e.name as exercise_name,
                  e.scoring_type, ml.reps, ml.sets, ml.weight_kg, ml.notes
           FROM movement_logs ml
           JOIN wod_movements wm ON wm.id = ml.movement_id
           JOIN exercises e ON e.id = wm.exercise_id
           JOIN section_logs sl ON sl.id = ml.section_log_id
           WHERE sl.workout_log_id = $1
           ORDER BY sl.id, ml.created_at"#,
    )
    .bind(workout_log_id)
    .fetch_all(pool)
    .await
}

/// Update a single movement log's reps, sets, weight, and notes (with user ownership check).
#[cfg(feature = "ssr")]
pub async fn update_movement_log_db(
    pool: &sqlx::PgPool,
    movement_log_id: uuid::Uuid,
    user_id: uuid::Uuid,
    reps: Option<i32>,
    sets: Option<i32>,
    weight_kg: Option<f32>,
    notes: Option<String>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE movement_logs ml
           SET reps = $2, sets = $3, weight_kg = $4, notes = $5
           FROM section_logs sl
           JOIN workout_logs wl ON wl.id = sl.workout_log_id
           WHERE ml.id = $1 AND ml.section_log_id = sl.id AND wl.user_id = $6"#,
    )
    .bind(movement_log_id)
    .bind(reps)
    .bind(sets)
    .bind(weight_kg)
    .bind(notes)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get per-set data for movement logs of a workout.
#[cfg(feature = "ssr")]
pub async fn get_movement_log_sets_db(
    pool: &sqlx::PgPool,
    workout_log_id: uuid::Uuid,
) -> Result<Vec<MovementLogSet>, sqlx::Error> {
    sqlx::query_as::<_, MovementLogSet>(
        r#"SELECT mls.id::text, mls.movement_log_id::text, mls.set_number,
                  mls.reps, mls.weight_kg, mls.notes,
                  mls.distance_meters, mls.calories
           FROM movement_log_sets mls
           JOIN movement_logs ml ON ml.id = mls.movement_log_id
           JOIN section_logs sl ON sl.id = ml.section_log_id
           WHERE sl.workout_log_id = $1
           ORDER BY mls.movement_log_id, mls.set_number"#,
    )
    .bind(workout_log_id)
    .fetch_all(pool)
    .await
}

/// Update a section log's score fields (with user ownership check).
#[cfg(feature = "ssr")]
#[allow(clippy::too_many_arguments)]
pub async fn update_section_log_db(
    pool: &sqlx::PgPool,
    section_log_id: uuid::Uuid,
    user_id: uuid::Uuid,
    finish_time_seconds: Option<i32>,
    rounds_completed: Option<i32>,
    extra_reps: Option<i32>,
    weight_kg: Option<f32>,
    is_rx: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE section_logs sl
           SET finish_time_seconds = $2, rounds_completed = $3, extra_reps = $4,
               weight_kg = $5, is_rx = $6
           FROM workout_logs wl
           WHERE sl.id = $1 AND sl.workout_log_id = wl.id AND wl.user_id = $7"#,
    )
    .bind(section_log_id)
    .bind(finish_time_seconds)
    .bind(rounds_completed)
    .bind(extra_reps)
    .bind(weight_kg)
    .bind(is_rx)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Update a single movement_log_set row (with user ownership check).
#[cfg(feature = "ssr")]
pub async fn update_movement_log_set_db(
    pool: &sqlx::PgPool,
    set_id: uuid::Uuid,
    user_id: uuid::Uuid,
    reps: Option<i32>,
    weight_kg: Option<f32>,
    distance_meters: Option<f32>,
    calories: Option<i32>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE movement_log_sets mls
           SET reps = $2, weight_kg = $3, distance_meters = $4, calories = $5
           FROM movement_logs ml
           JOIN section_logs sl ON sl.id = ml.section_log_id
           JOIN workout_logs wl ON wl.id = sl.workout_log_id
           WHERE mls.id = $1 AND mls.movement_log_id = ml.id AND wl.user_id = $6"#,
    )
    .bind(set_id)
    .bind(reps)
    .bind(weight_kg)
    .bind(distance_meters)
    .bind(calories)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Section leaderboard: top scores for a given section.
#[cfg(feature = "ssr")]
pub async fn section_leaderboard_db(
    pool: &sqlx::PgPool,
    section_id: uuid::Uuid,
    section_type: &str,
    limit: i64,
) -> Result<Vec<SectionLeaderboardEntry>, sqlx::Error> {
    // fortime: lower is better (ASC); amrap/emom/strength: higher is better (DESC)
    let order = if section_type == "fortime" {
        "ASC"
    } else {
        "DESC"
    };

    let query = format!(
        r#"SELECT u.display_name, u.avatar_url, sl.score_value,
                  sl.is_rx, sl.finish_time_seconds, sl.rounds_completed,
                  sl.extra_reps, sl.weight_kg
           FROM section_logs sl
           JOIN workout_logs wl ON wl.id = sl.workout_log_id
           JOIN users u ON u.id = wl.user_id
           WHERE sl.section_id = $1 AND sl.score_value IS NOT NULL
           ORDER BY sl.is_rx DESC, sl.score_value {}
           LIMIT $2"#,
        order
    );

    #[allow(clippy::type_complexity)]
    let rows: Vec<(
        String,
        Option<String>,
        i32,
        bool,
        Option<i32>,
        Option<i32>,
        Option<i32>,
        Option<f32>,
    )> = sqlx::query_as(&query)
        .bind(section_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(
            |(
                display_name,
                avatar_url,
                score_value,
                is_rx,
                finish_time_seconds,
                rounds_completed,
                extra_reps,
                weight_kg,
            )| {
                SectionLeaderboardEntry {
                    display_name,
                    avatar_url,
                    score_value,
                    is_rx,
                    finish_time_seconds,
                    rounds_completed,
                    extra_reps,
                    weight_kg,
                }
            },
        )
        .collect())
}

/// Personal best for a section (best score_value for the user).
#[cfg(feature = "ssr")]
pub async fn personal_best_for_section_db(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    section_id: uuid::Uuid,
    section_type: &str,
) -> Result<Option<PersonalBest>, sqlx::Error> {
    let order = if section_type == "fortime" {
        "ASC"
    } else {
        "DESC"
    };

    let query = format!(
        r#"SELECT sl.score_value, sl.is_rx, sl.created_at::text
           FROM section_logs sl
           JOIN workout_logs wl ON wl.id = sl.workout_log_id
           WHERE sl.section_id = $1 AND wl.user_id = $2 AND sl.score_value IS NOT NULL
           ORDER BY sl.is_rx DESC, sl.score_value {}
           LIMIT 1"#,
        order
    );

    let row: Option<(i32, bool, String)> = sqlx::query_as(&query)
        .bind(section_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|(score_value, is_rx, logged_at)| PersonalBest {
        score_value,
        is_rx,
        logged_at,
    }))
}

/// Check if a user has already logged a score for a WOD on a given date.
#[cfg(feature = "ssr")]
pub async fn has_wod_score_db(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    wod_id: uuid::Uuid,
    workout_date: &str,
) -> Result<Option<String>, sqlx::Error> {
    let date: chrono::NaiveDate = workout_date
        .parse()
        .map_err(|e| sqlx::Error::Protocol(format!("Invalid date: {}", e)))?;

    let row: Option<(String,)> = sqlx::query_as(
        r#"SELECT id::text FROM workout_logs
           WHERE user_id = $1 AND wod_id = $2 AND workout_date = $3"#,
    )
    .bind(user_id)
    .bind(wod_id)
    .bind(date)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id,)| id))
}

/// Get a WOD with its sections (for the log page).
#[cfg(feature = "ssr")]
pub async fn get_wod_with_sections_db(
    pool: &sqlx::PgPool,
    wod_id: uuid::Uuid,
) -> Result<(Wod, Vec<WodSection>), sqlx::Error> {
    let wod = sqlx::query_as::<_, Wod>(
        r#"SELECT id::text, title, description, workout_type,
                  time_cap_minutes, programmed_date::text, created_by::text
           FROM wods WHERE id = $1"#,
    )
    .bind(wod_id)
    .fetch_one(pool)
    .await?;

    let sections = list_wod_sections_db(pool, wod_id).await?;
    Ok((wod, sections))
}

// ---- Workout Exercise Models & Queries (Custom Logging) ----

/// A single set of an exercise in a custom workout log.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct WorkoutExercise {
    pub id: String,
    pub exercise_id: String,
    pub exercise_name: String,
    pub set_number: i32,
    pub reps: Option<i32>,
    pub weight_kg: Option<f32>,
    pub duration_seconds: Option<i32>,
    pub notes: Option<String>,
    pub distance_meters: Option<f32>,
    pub calories: Option<i32>,
    pub scoring_type: String,
}

/// Input for a single exercise set in a custom workout.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExerciseSetInput {
    pub exercise_id: String,
    pub set_number: i32,
    pub reps: Option<i32>,
    pub weight_kg: Option<f32>,
    pub duration_seconds: Option<i32>,
    pub notes: Option<String>,
    pub distance_meters: Option<f32>,
    pub calories: Option<i32>,
}

/// Submit a custom workout with exercises in one transaction.
#[cfg(feature = "ssr")]
pub async fn submit_custom_workout_db(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    workout_date: &str,
    notes: Option<&str>,
    exercises: &[ExerciseSetInput],
) -> Result<uuid::Uuid, sqlx::Error> {
    let date: chrono::NaiveDate = workout_date
        .parse()
        .map_err(|e| sqlx::Error::Protocol(format!("Invalid date: {}", e)))?;

    let mut tx = pool.begin().await?;

    let (log_id,): (uuid::Uuid,) = sqlx::query_as(
        r#"INSERT INTO workout_logs (user_id, workout_date, notes, is_rx)
           VALUES ($1, $2, $3, true)
           RETURNING id"#,
    )
    .bind(user_id)
    .bind(date)
    .bind(notes)
    .fetch_one(&mut *tx)
    .await?;

    for input in exercises {
        let exercise_id: uuid::Uuid = input
            .exercise_id
            .parse()
            .map_err(|e| sqlx::Error::Protocol(format!("Invalid exercise_id: {}", e)))?;

        sqlx::query(
            r#"INSERT INTO workout_exercises
               (workout_log_id, exercise_id, set_number, reps, weight_kg, duration_seconds, notes, distance_meters, calories)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        )
        .bind(log_id)
        .bind(exercise_id)
        .bind(input.set_number)
        .bind(input.reps)
        .bind(input.weight_kg)
        .bind(input.duration_seconds)
        .bind(&input.notes)
        .bind(input.distance_meters)
        .bind(input.calories)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(log_id)
}

/// Get exercises for a workout log (custom logging).
#[cfg(feature = "ssr")]
pub async fn list_workout_exercises_db(
    pool: &sqlx::PgPool,
    workout_log_id: uuid::Uuid,
) -> Result<Vec<WorkoutExercise>, sqlx::Error> {
    sqlx::query_as::<_, WorkoutExercise>(
        r#"SELECT we.id::text, we.exercise_id::text, e.name as exercise_name,
                  we.set_number, we.reps, we.weight_kg, we.duration_seconds, we.notes,
                  we.distance_meters, we.calories, e.scoring_type
           FROM workout_exercises we
           JOIN exercises e ON e.id = we.exercise_id
           WHERE we.workout_log_id = $1
           ORDER BY we.sort_order, we.set_number"#,
    )
    .bind(workout_log_id)
    .fetch_all(pool)
    .await
}

/// Update an existing custom workout: replace notes and exercises.
#[cfg(feature = "ssr")]
pub async fn update_custom_workout_db(
    pool: &sqlx::PgPool,
    log_id: uuid::Uuid,
    user_id: uuid::Uuid,
    workout_date: &str,
    notes: Option<&str>,
    exercises: &[ExerciseSetInput],
) -> Result<(), sqlx::Error> {
    let date: chrono::NaiveDate = workout_date
        .parse()
        .map_err(|e| sqlx::Error::Protocol(format!("Invalid date: {}", e)))?;

    let mut tx = pool.begin().await?;

    // Verify ownership
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM workout_logs WHERE id = $1 AND user_id = $2")
            .bind(log_id)
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await?;
    if count.0 == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    // Update log
    sqlx::query("UPDATE workout_logs SET workout_date = $1, notes = $2 WHERE id = $3")
        .bind(date)
        .bind(notes)
        .bind(log_id)
        .execute(&mut *tx)
        .await?;

    // Delete old exercises and re-insert
    sqlx::query("DELETE FROM workout_exercises WHERE workout_log_id = $1")
        .bind(log_id)
        .execute(&mut *tx)
        .await?;

    for input in exercises {
        let exercise_id: uuid::Uuid = input
            .exercise_id
            .parse()
            .map_err(|e| sqlx::Error::Protocol(format!("Invalid exercise_id: {}", e)))?;

        sqlx::query(
            r#"INSERT INTO workout_exercises
               (workout_log_id, exercise_id, set_number, reps, weight_kg, duration_seconds, notes, distance_meters, calories)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        )
        .bind(log_id)
        .bind(exercise_id)
        .bind(input.set_number)
        .bind(input.reps)
        .bind(input.weight_kg)
        .bind(input.duration_seconds)
        .bind(&input.notes)
        .bind(input.distance_meters)
        .bind(input.calories)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Delete a workout log and its exercises.
#[cfg(feature = "ssr")]
pub async fn delete_workout_log_db(
    pool: &sqlx::PgPool,
    log_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query("DELETE FROM workout_logs WHERE id = $1 AND user_id = $2")
        .bind(log_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }
    Ok(())
}
