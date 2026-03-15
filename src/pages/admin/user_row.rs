use crate::auth::{AuthUser, UserRole};
use leptos::prelude::*;

use super::ChangeUserRole;

#[component]
pub fn UserRow(user: AuthUser, change_action: ServerAction<ChangeUserRole>) -> impl IntoView {
    let uid = user.id.clone();
    let role_str = user.role.to_string();
    let user_ident = user.identifier().to_string();

    view! {
        <div class="user-row">
            <div class="user-avatar">{user.initials()}</div>
            <div class="user-info">
                <span class="user-name">{user.display_name}</span>
                <span class="user-email">{user_ident}</span>
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
}
