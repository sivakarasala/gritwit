use crate::auth::{AuthUser, UserRole};
use leptos::prelude::*;

#[server]
async fn list_all_users() -> Result<Vec<AuthUser>, ServerFnError> {
    crate::auth::session::require_role(UserRole::Admin).await?;
    let pool = crate::db::db().await?;
    crate::db::list_users_db(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn change_user_role(user_id: String, new_role: String) -> Result<(), ServerFnError> {
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

#[component]
pub fn AdminPage() -> impl IntoView {
    let change_action = ServerAction::<ChangeUserRole>::new();
    let users = Resource::new(move || change_action.version().get(), |_| list_all_users());

    view! {
        <div class="admin-page">
            <Suspense fallback=|| view! { <p class="loading">"Loading users..."</p> }>
                {move || {
                    users.get().map(|result| {
                        match result {
                            Ok(list) => view! {
                                <div class="users-list">
                                    {list.into_iter().map(|user| {
                                        let uid = user.id.clone();
                                        let role_str = user.role.to_string();
                                        view! {
                                            <div class="user-row">
                                                <div class="user-avatar">{user.initials()}</div>
                                                <div class="user-info">
                                                    <span class="user-name">{user.display_name}</span>
                                                    <span class="user-email">{user.email}</span>
                                                </div>
                                                <div class="user-role-controls">
                                                    <span class={format!("role-badge role-badge--{}", role_str)}>{role_str.to_uppercase()}</span>
                                                    {(user.role != UserRole::Admin).then(|| {
                                                        let uid_promote = uid.clone();
                                                        let uid_demote = uid.clone();
                                                        let is_coach = user.role == UserRole::Coach;
                                                        view! {
                                                            <div class="role-actions">
                                                                {(!is_coach).then(|| {
                                                                    view! {
                                                                        <button
                                                                            class="role-btn role-btn--promote"
                                                                            disabled=move || change_action.pending().get()
                                                                            on:click=move |_| {
                                                                                change_action.dispatch(ChangeUserRole {
                                                                                    user_id: uid_promote.clone(),
                                                                                    new_role: "coach".to_string(),
                                                                                });
                                                                            }
                                                                        >"Make Coach"</button>
                                                                    }
                                                                })}
                                                                {is_coach.then(|| {
                                                                    view! {
                                                                        <button
                                                                            class="role-btn role-btn--demote"
                                                                            disabled=move || change_action.pending().get()
                                                                            on:click=move |_| {
                                                                                change_action.dispatch(ChangeUserRole {
                                                                                    user_id: uid_demote.clone(),
                                                                                    new_role: "athlete".to_string(),
                                                                                });
                                                                            }
                                                                        >"Demote"</button>
                                                                    }
                                                                })}
                                                            </div>
                                                        }
                                                    })}
                                                </div>
                                            </div>
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
