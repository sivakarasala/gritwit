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

#[server]
async fn update_gender(gender: String) -> Result<(), ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::update_user_gender_db(&pool, user_uuid, &gender)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
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
                                let current_gender = data.user.gender.clone().unwrap_or_default();
                                let gender_signal = RwSignal::new(current_gender);
                                let gender_saving = RwSignal::new(false);
                                let gender_saved = RwSignal::new(false);

                                let on_gender_change = move |ev: leptos::ev::Event| {
                                    let val = event_target_value(&ev);
                                    gender_signal.set(val.clone());
                                    gender_saving.set(true);
                                    gender_saved.set(false);
                                    leptos::task::spawn_local(async move {
                                        let _ = update_gender(val).await;
                                        gender_saving.set(false);
                                        gender_saved.set(true);
                                        set_timeout(move || gender_saved.set(false), std::time::Duration::from_secs(2));
                                    });
                                };

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
                                            <span class="profile-action-label">"Gender"</span>
                                            <div class="profile-gender-control">
                                                <select
                                                    class="profile-gender-select"
                                                    on:change=on_gender_change
                                                    prop:value=move || gender_signal.get()
                                                >
                                                    <option value="" selected=move || gender_signal.get().is_empty()>"Not set"</option>
                                                    <option value="male" selected=move || gender_signal.get() == "male">"Male"</option>
                                                    <option value="female" selected=move || gender_signal.get() == "female">"Female"</option>
                                                </select>
                                                {move || gender_saving.get().then(|| view! {
                                                    <span class="profile-gender-status saving">"Saving..."</span>
                                                })}
                                                {move || gender_saved.get().then(|| view! {
                                                    <span class="profile-gender-status saved">"Saved"</span>
                                                })}
                                            </div>
                                        </div>
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
