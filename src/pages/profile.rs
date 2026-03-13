use crate::auth::AuthUser;
use leptos::prelude::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProfileData {
    pub user: AuthUser,
    pub workouts: i64,
    pub streak: i64,
}

#[server]
async fn get_profile() -> Result<ProfileData, ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;

    let workouts = crate::db::count_workouts_db(&pool, user_uuid)
        .await
        .unwrap_or(0);
    let streak = crate::db::streak_days_db(&pool, user_uuid)
        .await
        .unwrap_or(0);

    Ok(ProfileData {
        user,
        workouts,
        streak,
    })
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let profile = Resource::new(|| (), |_| get_profile());

    view! {
        <div class="profile-page">
            <Suspense fallback=|| view! { <p class="loading">"Loading..."</p> }>
                {move || {
                    profile.get().map(|result| {
                        match result {
                            Ok(data) => {
                                let role_str = data.user.role.to_string();
                                view! {
                                    <div class="profile-card">
                                        <div class="profile-avatar">
                                            {if let Some(ref url) = data.user.avatar_url {
                                                view! { <img src={url.clone()} class="profile-avatar-img" referrerpolicy="no-referrer"/> }.into_any()
                                            } else {
                                                view! { <span class="profile-avatar-initials">{data.user.initials()}</span> }.into_any()
                                            }}
                                        </div>
                                        <h2 class="profile-name">{data.user.display_name}</h2>
                                        <p class="profile-email">{data.user.email}</p>
                                        <span class={format!("role-badge role-badge--{}", role_str)}>{role_str.to_uppercase()}</span>
                                    </div>

                                    <div class="profile-stats">
                                        <div class="profile-stat">
                                            <span class="profile-stat-num">{data.workouts}</span>
                                            <span class="profile-stat-label">"Workouts"</span>
                                        </div>
                                        <div class="profile-stat">
                                            <span class="profile-stat-num">{data.streak}</span>
                                            <span class="profile-stat-label">"Day Streak"</span>
                                        </div>
                                    </div>

                                    <div class="profile-actions">
                                        <div class="profile-action-row">
                                            <span class="profile-action-label">"Theme"</span>
                                            <button class="theme-toggle" on:click=move |_| {
                                                #[cfg(feature = "hydrate")]
                                                { crate::voice::toggle_theme(); }
                                            }>
                                                <span class="theme-icon theme-icon--sun"></span>
                                                <span class="theme-icon theme-icon--moon"></span>
                                            </button>
                                        </div>
                                        <a href="/auth/logout" rel="external" class="profile-logout-btn">"Sign Out"</a>
                                    </div>
                                }.into_any()
                            }
                            Err(e) => view! { <p class="error">{format!("Error: {}", e)}</p> }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
