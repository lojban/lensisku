use super::dto::{NotificationListResponse, SubscriptionResponse, SubscriptionState};
use super::models::{Notification, Subscription, SubscriptionTrigger};
use deadpool_postgres::Pool;

pub async fn subscribe_to_valsi(
    pool: &Pool,
    user_id: i32,
    valsi_id: i32,
    trigger_type: SubscriptionTrigger,
) -> Result<SubscriptionResponse, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    // Check if valsi exists
    let valsi_exists = client
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM valsi WHERE valsiid = $1)",
            &[&valsi_id],
        )
        .await?
        .get::<_, bool>(0);

    if !valsi_exists {
        return Ok(SubscriptionResponse {
            success: false,
            message: "Valsi not found".to_string(),
        });
    }

    client
        .execute(
            "INSERT INTO valsi_subscriptions (valsi_id, user_id, trigger_type)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (valsi_id, user_id, trigger_type)
                 DO UPDATE SET unsubscribed = false, unsubscribed_at = NULL",
            &[&valsi_id, &user_id, &trigger_type],
        )
        .await?;

    Ok(SubscriptionResponse {
        success: true,
        message: "Successfully subscribed to valsi".to_string(),
    })
}

pub async fn unsubscribe_from_valsi(
    pool: &Pool,
    user_id: i32,
    valsi_id: i32,
    trigger_type: SubscriptionTrigger,
) -> Result<SubscriptionResponse, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let affected_rows = client
        .execute(
            "UPDATE valsi_subscriptions
                     SET unsubscribed = true, unsubscribed_at = CURRENT_TIMESTAMP
                     WHERE valsi_id = $1 AND user_id = $2 AND trigger_type = $3 AND NOT unsubscribed",
            &[&valsi_id, &user_id, &trigger_type],
        )
        .await?;

    if affected_rows == 0 {
        return Ok(SubscriptionResponse {
            success: true,
            message: "No active subscription found".to_string(),
        });
    }

    Ok(SubscriptionResponse {
        success: true,
        message: "Successfully unsubscribed from valsi".to_string(),
    })
}

pub async fn get_user_subscriptions(
    pool: &Pool,
    user_id: i32,
) -> Result<Vec<Subscription>, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let rows = client
        .query(
            "SELECT * FROM valsi_subscriptions
             WHERE user_id = $1 AND NOT unsubscribed
             ORDER BY created_at DESC",
            &[&user_id],
        )
        .await?;

    Ok(rows.into_iter().map(Subscription::from).collect())
}

pub async fn get_user_notifications(
    pool: &Pool,
    user_id: i32,
    page: i64,
    per_page: i64,
    unread_only: bool,
) -> Result<NotificationListResponse, Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    let offset = (page - 1) * per_page;

    let unread_condition = if unread_only {
        "AND read_at IS NULL"
    } else {
        ""
    };

    let rows = client
        .query(
            &format!(
                "SELECT * FROM user_notifications
                WHERE user_id = $1 {}
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3",
                unread_condition
            ),
            &[&user_id, &per_page, &offset],
        )
        .await?;

    let notifications: Vec<Notification> = rows.into_iter().map(Notification::from).collect();

    // Get total count
    let total: i64 = client
        .query_one(
            &format!(
                "SELECT COUNT(*) FROM user_notifications
                WHERE user_id = $1 {}",
                unread_condition
            ),
            &[&user_id],
        )
        .await?
        .get(0);

    // Get unread count
    let unread_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM user_notifications
             WHERE user_id = $1 AND read_at IS NULL",
            &[&user_id],
        )
        .await?
        .get(0);

    Ok(NotificationListResponse {
        notifications,
        total,
        unread_count,
    })
}

pub async fn mark_notifications_read(
    pool: &Pool,
    user_id: i32,
    notification_ids: Option<Vec<i32>>,
) -> Result<i64, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let query = if let Some(ids) = notification_ids {
        client
            .execute(
                "UPDATE user_notifications
                 SET read_at = CURRENT_TIMESTAMP
                 WHERE user_id = $1 AND notification_id = ANY($2)
                 AND read_at IS NULL",
                &[&user_id, &ids],
            )
            .await?
    } else {
        client
            .execute(
                "UPDATE user_notifications
                 SET read_at = CURRENT_TIMESTAMP
                 WHERE user_id = $1 AND read_at IS NULL",
                &[&user_id],
            )
            .await?
    };

    Ok(query as i64)
}

pub async fn get_subscription_state(
    pool: &Pool,
    user_id: i32,
    valsi_id: i32,
) -> Result<SubscriptionState, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let rows = client
        .query(
            "SELECT trigger_type 
             FROM valsi_subscriptions 
             WHERE user_id = $1 AND valsi_id = $2 
             AND NOT unsubscribed",
            &[&user_id, &valsi_id],
        )
        .await?;

    let subscriptions: Vec<SubscriptionTrigger> =
        rows.iter().map(|row| row.get("trigger_type")).collect();

    Ok(SubscriptionState {
        is_subscribed: !subscriptions.is_empty(),
        subscriptions,
    })
}
