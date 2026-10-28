use crate::error::AppError;
use chrono::Utc;
use deadpool_postgres::Pool;

use crate::middleware::limiter::EmailConfirmationLimiter;

use super::models::UserRole;

pub async fn confirm_email(
    pool: &Pool,
    token: &str,
    email_limiter: &EmailConfirmationLimiter,
) -> Result<(), AppError> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Find user with this token
    let user = transaction
        .query_opt(
            "SELECT userid, email, email_confirmation_sent_at 
             FROM users 
             WHERE email_confirmation_token = $1
             AND email_confirmed = false",
            &[&token],
        )
        .await?
        .ok_or("Invalid or expired token")?;

    let email: String = user.get("email");

    // Check rate limit
    if !email_limiter.check_rate_limit(&email).await? {
        return Err(AppError::BadRequest(
            "Too many confirmation attempts. Please try again later.".into(),
        ));
    }

    // Check token age (24 hour expiry)
    let sent_at: chrono::DateTime<Utc> = user.get("email_confirmation_sent_at");
    if Utc::now().signed_duration_since(sent_at).num_hours() > 24 {
        return Err(AppError::BadRequest(
            "Confirmation token has expired".into(),
        ));
    }

    // Update user
    transaction
        .execute(
            "UPDATE users 
             SET email_confirmed = true,
                 email_confirmation_token = NULL,
                 role = $1
             WHERE userid = $2",
            &[&UserRole::Editor.to_string(), &user.get::<_, i32>("userid")],
        )
        .await?;

    transaction.commit().await?;
    Ok(())
}
