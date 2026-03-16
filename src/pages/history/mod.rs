mod history_card;

use crate::components::DeleteModal;
use crate::db::{
    MovementLogSet, MovementLogWithName, SectionScoreWithMeta, WorkoutExercise, WorkoutLog,
};
use crate::pages::wod::week_calendar::WeeklyCalendar;
use history_card::HistoryCard;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// A workout log enriched with exercise details and optional WOD title.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct HistoryEntry {
    pub log: WorkoutLog,
    pub wod_title: Option<String>,
    pub exercises: Vec<WorkoutExercise>,
    pub section_scores: Vec<SectionScoreWithMeta>,
    pub movement_logs: Vec<MovementLogWithName>,
    pub movement_log_sets: Vec<MovementLogSet>,
}

#[server]
async fn delete_history_entry(log_id: String) -> Result<(), ServerFnError> {
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

#[server]
pub(crate) async fn update_movement_log(
    movement_log_id: String,
    reps: Option<i32>,
    sets: Option<i32>,
    weight_kg: Option<f32>,
    notes: Option<String>,
) -> Result<(), ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let ml_uuid: uuid::Uuid = movement_log_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::update_movement_log_db(&pool, ml_uuid, user_uuid, reps, sets, weight_kg, notes)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[server]
pub(crate) async fn update_movement_log_set(
    set_id: String,
    reps: Option<i32>,
    weight_kg: Option<f32>,
) -> Result<(), ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let set_uuid: uuid::Uuid = set_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::update_movement_log_set_db(&pool, set_uuid, user_uuid, reps, weight_kg)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[server]
pub(crate) async fn update_section_score(
    section_log_id: String,
    finish_time_seconds: Option<i32>,
    rounds_completed: Option<i32>,
    extra_reps: Option<i32>,
    weight_kg: Option<f32>,
    is_rx: bool,
) -> Result<(), ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let sl_uuid: uuid::Uuid = section_log_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::update_section_log_db(
        &pool,
        sl_uuid,
        user_uuid,
        finish_time_seconds,
        rounds_completed,
        extra_reps,
        weight_kg,
        is_rx,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[server]
async fn get_history_for_date(date: String) -> Result<Vec<HistoryEntry>, ServerFnError> {
    use chrono::NaiveDate;

    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;

    let parsed: NaiveDate = date
        .parse()
        .map_err(|e| ServerFnError::new(format!("Invalid date: {}", e)))?;

    let logs = crate::db::list_workouts_by_date_db(&pool, user_uuid, parsed)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut entries = Vec::with_capacity(logs.len());
    for log in logs {
        // Get WOD title if this is a WOD workout
        let wod_title = if let Some(ref wod_id) = log.wod_id {
            let uuid: uuid::Uuid = wod_id
                .parse()
                .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
            let title: Option<(String,)> = sqlx::query_as("SELECT title FROM wods WHERE id = $1")
                .bind(uuid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;
            title.map(|(t,)| t)
        } else {
            None
        };

        let log_uuid: uuid::Uuid = log
            .id
            .parse()
            .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;

        // Get exercises for custom workouts
        let exercises = crate::db::list_workout_exercises_db(&pool, log_uuid)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        // Get section scores, movement logs, and per-set data for WOD workouts
        let (section_scores, movement_logs, movement_log_sets) = if log.wod_id.is_some() {
            let (scores, movements, sets) = tokio::join!(
                crate::db::get_section_scores_with_meta_db(&pool, log_uuid),
                crate::db::get_movement_logs_with_names_db(&pool, log_uuid),
                crate::db::get_movement_log_sets_db(&pool, log_uuid),
            );
            (
                scores.map_err(|e| ServerFnError::new(e.to_string()))?,
                movements.map_err(|e| ServerFnError::new(e.to_string()))?,
                sets.map_err(|e| ServerFnError::new(e.to_string()))?,
            )
        } else {
            (vec![], vec![], vec![])
        };

        entries.push(HistoryEntry {
            log,
            wod_title,
            exercises,
            section_scores,
            movement_logs,
            movement_log_sets,
        });
    }

    Ok(entries)
}

#[component]
pub fn HistoryPage() -> impl IntoView {
    // Use ?date= query param if present, otherwise default to today
    let params = leptos_router::hooks::use_query_map();
    let initial_date = {
        let d = params
            .read_untracked()
            .get("date")
            .unwrap_or_default()
            .to_string();
        if d.is_empty() {
            crate::pages::wod::week_calendar::today_iso()
        } else {
            d
        }
    };
    let selected_date = RwSignal::new(initial_date);
    let is_loading = RwSignal::new(false);

    let delete_action = ServerAction::<DeleteHistoryEntry>::new();
    let show_delete = RwSignal::new(false);
    let pending_delete_log_id = RwSignal::new(String::new());

    let history = Resource::new(
        move || (selected_date.get(), delete_action.version().get()),
        move |(date, _)| async move {
            is_loading.set(true);
            let result = if date.is_empty() {
                Ok(vec![])
            } else {
                get_history_for_date(date).await
            };
            is_loading.set(false);
            result
        },
    );

    view! {
        <div class="history-page">
            <WeeklyCalendar selected_date=selected_date/>
            <div class="history-loading-bar" class:active=move || is_loading.get()></div>
            <Transition fallback=|| view! { <p class="loading">"Loading..."</p> }>
                {move || {
                    history.get().map(|result| match result {
                        Ok(entries) if entries.is_empty() => view! {
                            <div class="empty-state">
                                <p class="empty-title">"No workouts"</p>
                                <p class="empty-sub">"Nothing logged for this day."</p>
                            </div>
                        }
                        .into_any(),
                        Ok(entries) => view! {
                            <div class="results-feed">
                                {entries
                                    .into_iter()
                                    .map(|entry| {
                                        view! {
                                            <HistoryCard
                                                entry=entry
                                                show_delete=show_delete
                                                pending_delete_log_id=pending_delete_log_id
                                            />
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        }
                        .into_any(),
                        Err(e) => {
                            view! { <p class="error">{format!("Error: {}", e)}</p> }.into_any()
                        }
                    })
                }}
            </Transition>
            <DeleteModal
                show=show_delete
                title="Delete this workout?"
                subtitle="This will permanently remove the log and all its data. This cannot be undone."
                confirm_label="Delete"
                on_confirm=move || {
                    delete_action.dispatch(DeleteHistoryEntry {
                        log_id: pending_delete_log_id.get_untracked(),
                    });
                }
            />
        </div>
    }
}
