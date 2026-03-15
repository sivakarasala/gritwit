use leptos::prelude::*;

/// Send an OTP to the given phone number.
/// In dev mode (no SMS config), generates a code locally and logs it.
#[server]
pub async fn send_otp(phone: String) -> Result<(), ServerFnError> {
    let phone = phone.trim().to_string();

    // Basic validation: must start with + and be 10-15 digits
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if !phone.starts_with('+') || digits.len() < 10 || digits.len() > 15 {
        return Err(ServerFnError::new(
            "Please enter a valid phone number with country code (e.g. +919876543210)",
        ));
    }

    // Rate limit: max 1 OTP per phone per 30 seconds
    let pool = crate::db::db().await?;
    let recent: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM otp_codes WHERE phone = $1 AND created_at > now() - interval '30 seconds'",
    )
    .bind(&phone)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if recent.map(|r| r.0).unwrap_or(0) > 0 {
        return Err(ServerFnError::new(
            "Please wait 30 seconds before requesting another OTP",
        ));
    }

    let config =
        crate::configuration::get_configuration().map_err(|e| ServerFnError::new(e.to_string()))?;

    // Clean up expired OTP codes
    sqlx::query("DELETE FROM otp_codes WHERE expires_at < now()")
        .execute(&pool)
        .await
        .ok();

    let code = generate_otp();
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);

    // Store OTP in database
    sqlx::query("INSERT INTO otp_codes (phone, code, expires_at) VALUES ($1, $2, $3)")
        .bind(&phone)
        .bind(&code)
        .bind(expires_at)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Send via 2Factor.in or log in dev mode
    if let Some(ref sms) = config.sms {
        use secrecy::ExposeSecret;
        send_2factor_otp(&phone, &code, sms.api_key.expose_secret()).await?;
    } else {
        tracing::warn!("DEV MODE: OTP for {} is {}", phone, code);
    }

    Ok(())
}

/// Verify OTP and log the user in. Creates account if new phone.
#[server]
pub async fn verify_otp(phone: String, code: String) -> Result<super::OtpResult, ServerFnError> {
    let phone = phone.trim().to_string();
    let code = code.trim().to_string();

    if code.len() != 6 {
        return Err(ServerFnError::new("Please enter the 6-digit code"));
    }

    let pool = crate::db::db().await?;

    // Atomically find and mark OTP as verified
    let otp: Option<(uuid::Uuid,)> = sqlx::query_as(
        r#"UPDATE otp_codes SET verified = true
           WHERE id = (
               SELECT id FROM otp_codes
               WHERE phone = $1 AND code = $2 AND expires_at > now() AND verified = false
               ORDER BY created_at DESC LIMIT 1
           )
           RETURNING id"#,
    )
    .bind(&phone)
    .bind(&code)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let Some(otp_row) = otp else {
        return Err(ServerFnError::new("Invalid or expired OTP"));
    };

    // Clean up old/expired codes
    sqlx::query("DELETE FROM otp_codes WHERE phone = $1 AND id != $2")
        .bind(&phone)
        .bind(otp_row.0)
        .execute(&pool)
        .await
        .ok();

    // Find or create user
    let existing: Option<(uuid::Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE phone = $1")
        .bind(&phone)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let (user_id, is_new) = if let Some(row) = existing {
        (row.0, false)
    } else {
        let role = super::default_role_for_new_user(&pool).await?;

        let row: (uuid::Uuid,) = sqlx::query_as(
            r#"INSERT INTO users (phone, display_name, role)
               VALUES ($1, $2, $3::user_role)
               RETURNING id"#,
        )
        .bind(&phone)
        .bind("Athlete")
        .bind(role)
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        (row.0, true)
    };

    // Set session
    let session = super::session::get_session().await?;
    super::session::set_user_id(&session, &user_id.to_string()).await?;

    Ok(if is_new {
        super::OtpResult::NewAccount
    } else {
        super::OtpResult::Existing
    })
}

#[cfg(feature = "ssr")]
fn generate_otp() -> String {
    use argon2::password_hash::rand_core::{OsRng, RngCore};
    let code = OsRng.next_u32() % 1_000_000;
    format!("{:06}", code)
}

#[cfg(feature = "ssr")]
async fn send_2factor_otp(phone: &str, otp: &str, api_key: &str) -> Result<(), ServerFnError> {
    let phone_num = phone.trim_start_matches('+');

    let url = format!(
        "https://2factor.in/API/V1/{}/SMS/{}/{}/GrndItOTP",
        api_key, phone_num, otp
    );

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to send OTP: {}", e)))?;

    let text = resp.text().await.unwrap_or_default();
    tracing::info!("2Factor response: {}", text);

    if text.contains("Error") || text.contains("error") {
        tracing::error!("2Factor error: {}", text);
        return Err(ServerFnError::new("Failed to send OTP. Please try again."));
    }

    Ok(())
}
