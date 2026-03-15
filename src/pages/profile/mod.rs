use crate::auth::{clean_error, AuthUser};
use crate::components::{SelectOption, SingleSelect};
use leptos::prelude::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProfileData {
    pub user: AuthUser,
    pub workouts: i64,
    pub streak: i64,
    pub has_password: bool,
}

#[server]
async fn get_profile() -> Result<ProfileData, ServerFnError> {
    let (user, pool, user_uuid) = crate::auth::session::auth_context().await?;

    let workouts = crate::db::count_workouts_db(&pool, user_uuid)
        .await
        .unwrap_or(0);
    let streak = crate::db::streak_days_db(&pool, user_uuid)
        .await
        .unwrap_or(0);

    let has_password: (bool,) =
        sqlx::query_as("SELECT password_hash IS NOT NULL FROM users WHERE id = $1")
            .bind(user_uuid)
            .fetch_one(&pool)
            .await
            .unwrap_or((false,));

    Ok(ProfileData {
        user,
        workouts,
        streak,
        has_password: has_password.0,
    })
}

#[server]
async fn update_profile(
    display_name: String,
    email: String,
    phone: String,
    gender: String,
) -> Result<(), ServerFnError> {
    let (_user, pool, user_uuid) = crate::auth::session::auth_context().await?;

    let name = crate::auth::validate_name(&display_name)?;
    let email_val = crate::auth::validate_email(&email)?;
    let email_opt = email_val.as_deref();

    let phone_val = phone.trim().to_string();
    let phone_opt = if phone_val.is_empty() {
        None
    } else {
        Some(phone_val.as_str())
    };

    let gender_opt = if gender.is_empty() {
        None
    } else {
        Some(gender.as_str())
    };

    crate::db::update_user_profile_db(&pool, user_uuid, &name, email_opt, phone_opt, gender_opt)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("unique") || msg.contains("duplicate") {
                if msg.contains("email") {
                    ServerFnError::new("This email is already linked to another account")
                } else if msg.contains("phone") {
                    ServerFnError::new("This phone number is already linked to another account")
                } else {
                    ServerFnError::new("This value is already in use by another account")
                }
            } else {
                ServerFnError::new("Failed to update profile")
            }
        })?;

    Ok(())
}

#[server]
async fn set_password(password: String) -> Result<(), ServerFnError> {
    let (user, pool, user_uuid) = crate::auth::session::auth_context().await?;

    crate::auth::validate_password(&password)?;

    // Require email to be set before setting password
    if user.email.is_none() {
        return Err(ServerFnError::new(
            "Please add your email in profile first, then set a password",
        ));
    }

    let hash = crate::auth::hash_password(&password)?;

    sqlx::query("UPDATE users SET password_hash = $1, updated_at = now() WHERE id = $2")
        .bind(&hash)
        .bind(user_uuid)
        .execute(&pool)
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
                                view! { <ProfileContent data=data/> }.into_any()
                            }
                            Err(e) => view! { <p class="error">{format!("Error: {}", e)}</p> }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn ProfileContent(data: ProfileData) -> impl IntoView {
    let role_str = data.user.role.to_string();

    // Editable fields
    let name = RwSignal::new(data.user.display_name.clone());
    let email = RwSignal::new(data.user.email.clone().unwrap_or_default());
    let phone = RwSignal::new(data.user.phone.clone().unwrap_or_default());
    let gender = RwSignal::new(data.user.gender.clone().unwrap_or_default());

    let profile_saving = RwSignal::new(false);
    let profile_saved = RwSignal::new(false);
    let profile_error = RwSignal::new(Option::<String>::None);

    // Password fields
    let new_password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());
    let pw_saving = RwSignal::new(false);
    let pw_result = RwSignal::new(Option::<Result<(), String>>::None);
    let has_password = RwSignal::new(data.has_password);

    let on_save_profile = move |_| {
        profile_saving.set(true);
        profile_saved.set(false);
        profile_error.set(None);
        let n = name.get_untracked();
        let e = email.get_untracked();
        let p = phone.get_untracked();
        let g = gender.get_untracked();
        leptos::task::spawn_local(async move {
            match update_profile(n, e, p, g).await {
                Ok(_) => {
                    profile_saving.set(false);
                    profile_saved.set(true);
                    set_timeout(
                        move || profile_saved.set(false),
                        std::time::Duration::from_secs(2),
                    );
                }
                Err(e) => {
                    profile_saving.set(false);
                    profile_error.set(Some(clean_error(&e)));
                }
            }
        });
    };

    let on_set_password = move |_| {
        let pw = new_password.get_untracked();
        let confirm = confirm_password.get_untracked();
        if pw != confirm {
            pw_result.set(Some(Err("Passwords do not match".to_string())));
            return;
        }
        pw_saving.set(true);
        pw_result.set(None);
        leptos::task::spawn_local(async move {
            match set_password(pw).await {
                Ok(_) => {
                    pw_saving.set(false);
                    pw_result.set(Some(Ok(())));
                    new_password.set(String::new());
                    confirm_password.set(String::new());
                    has_password.set(true);
                    set_timeout(
                        move || pw_result.set(None),
                        std::time::Duration::from_secs(3),
                    );
                }
                Err(e) => {
                    pw_saving.set(false);
                    pw_result.set(Some(Err(clean_error(&e))));
                }
            }
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
            <h2 class="profile-name">{data.user.display_name.clone()}</h2>
            <p class="profile-email">{data.user.identifier().to_string()}</p>
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
            <div class="profile-edit-section">
                <div class="profile-field">
                    <label class="profile-field-label">"Name"</label>
                    <input
                        type="text"
                        class="profile-input"
                        maxlength="100"
                        prop:value=move || name.get()
                        on:input=move |ev| name.set(event_target_value(&ev))
                    />
                </div>
                <div class="profile-field">
                    <label class="profile-field-label">"Email"</label>
                    <input
                        type="email"
                        class="profile-input"
                        placeholder="your@email.com"
                        prop:value=move || email.get()
                        on:input=move |ev| email.set(event_target_value(&ev))
                    />
                </div>
                <div class="profile-field">
                    <label class="profile-field-label">"Phone"</label>
                    <input
                        type="tel"
                        class="profile-input"
                        placeholder="+919876543210"
                        prop:value=move || phone.get()
                        on:input=move |ev| phone.set(event_target_value(&ev))
                    />
                </div>
                <div class="profile-field">
                    <label class="profile-field-label">"Gender"</label>
                    <SingleSelect
                        options=vec![
                            SelectOption { value: "".to_string(), label: "Not set".to_string() },
                            SelectOption { value: "male".to_string(), label: "Male".to_string() },
                            SelectOption { value: "female".to_string(), label: "Female".to_string() },
                        ]
                        selected=gender
                        placeholder="Not set"
                    />
                </div>

                {move || profile_error.get().map(|e| view! {
                    <p class="profile-error">{e}</p>
                })}

                <button
                    class="profile-save-btn"
                    class:btn--loading=move || profile_saving.get()
                    class:btn--success=move || profile_saved.get()
                    disabled=move || profile_saving.get()
                    on:click=on_save_profile
                >
                    {move || if profile_saved.get() {
                        "\u{2713} Saved!".to_string()
                    } else if profile_saving.get() {
                        "Saving...".to_string()
                    } else {
                        "Save Profile".to_string()
                    }}
                </button>
            </div>

            <div class="profile-divider"></div>

            <div class="profile-edit-section">
                <h3 class="profile-section-title">
                    {move || if has_password.get() { "Update Password" } else { "Set Password" }}
                </h3>
                <p class="profile-section-hint">
                    {move || if has_password.get() {
                        "Change your existing password"
                    } else {
                        "Set a password to enable email + password login"
                    }}
                </p>
                <div class="profile-field">
                    <input
                        type="password"
                        class="profile-input"
                        placeholder="New password (min 8 chars)"
                        minlength="8"
                        prop:value=move || new_password.get()
                        on:input=move |ev| new_password.set(event_target_value(&ev))
                    />
                </div>
                <div class="profile-field">
                    <input
                        type="password"
                        class="profile-input"
                        placeholder="Confirm password"
                        prop:value=move || confirm_password.get()
                        on:input=move |ev| confirm_password.set(event_target_value(&ev))
                    />
                </div>

                {move || pw_result.get().and_then(|r| r.err()).map(|e| view! {
                    <p class="profile-error">{e}</p>
                })}

                <button
                    class="profile-save-btn profile-save-btn--secondary"
                    class:btn--loading=move || pw_saving.get()
                    class:btn--success=move || matches!(pw_result.get(), Some(Ok(())))
                    disabled=move || pw_saving.get() || new_password.get().is_empty() || confirm_password.get().is_empty()
                    on:click=on_set_password
                >
                    {move || if matches!(pw_result.get(), Some(Ok(()))) {
                        "\u{2713} Password Saved!".to_string()
                    } else if pw_saving.get() {
                        "Saving...".to_string()
                    } else if has_password.get() {
                        "Update Password".to_string()
                    } else {
                        "Set Password".to_string()
                    }}
                </button>
            </div>

            <div class="profile-divider"></div>

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
    }
}
