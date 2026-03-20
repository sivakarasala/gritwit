mod exercise_table;
mod user_row;

use crate::auth::AuthUser;
#[cfg(feature = "ssr")]
use crate::auth::UserRole;
use exercise_table::ExerciseTable;
use leptos::prelude::*;
use user_row::UserRow;

#[server]
async fn list_all_users() -> Result<Vec<AuthUser>, ServerFnError> {
    crate::auth::session::require_role(UserRole::Admin).await?;
    let pool = crate::db::db().await?;
    crate::db::list_users_db(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn change_user_role(user_id: String, new_role: String) -> Result<(), ServerFnError> {
    crate::auth::session::require_role(UserRole::Admin).await?;
    let pool = crate::db::db().await?;
    let uid: uuid::Uuid = user_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    if !["athlete", "coach", "admin"].contains(&new_role.as_str()) {
        return Err(ServerFnError::new("Invalid role"));
    }
    crate::db::update_user_role_db(&pool, uid, &new_role)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[derive(Clone, Copy, PartialEq)]
enum AdminTab {
    Users,
    Exercises,
}

#[component]
fn AdminNav(tab: AdminTab) -> impl IntoView {
    view! {
        <nav class="admin-tabs">
            <a
                href="/admin"
                class="admin-tab"
                class:admin-tab--active=move || tab == AdminTab::Users
            >"Users"</a>
            <a
                href="/admin/exercises"
                class="admin-tab"
                class:admin-tab--active=move || tab == AdminTab::Exercises
            >"Exercises"</a>
        </nav>
    }
}

#[component]
pub fn AdminPage() -> impl IntoView {
    let change_action = ServerAction::<ChangeUserRole>::new();
    let users = Resource::new(move || change_action.version().get(), |_| list_all_users());

    view! {
        <div class="admin-page">
            <AdminNav tab=AdminTab::Users/>
            <Suspense fallback=|| view! { <p class="loading">"Loading users..."</p> }>
                {move || {
                    users.get().map(|result| {
                        match result {
                            Ok(list) => view! {
                                <div class="users-list">
                                    {list.into_iter().map(|user| {
                                        view! {
                                            <UserRow user=user change_action=change_action/>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any(),
                            Err(e) => view! { <p class="error">{format!("Error: {}", e)}</p> }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
pub fn AdminExercisesPage() -> impl IntoView {
    view! {
        <div class="admin-page admin-page--wide">
            <AdminNav tab=AdminTab::Exercises/>
            <ExerciseTable/>
        </div>
    }
}
