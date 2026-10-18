use bytes::BytesMut;
use deadpool_postgres::Pool;
use std::error::Error;
use tokio_postgres::types::{FromSql, IsNull, ToSql, Type};

use super::dto; // For PayPalSubscriptionDto, SubscriptionWebhookEventResource etc.
use super::paypal; // For event constants
use super::{
    dto::{BalanceResponse, CreatePaymentRequest},
    error::PaymentError,
    paypal::PayPalOrder,
};
use chrono::{DateTime, Utc}; // For parsing timestamps
use log::{debug, error, info, warn}; // For logging

#[derive(Debug)]
pub enum PaymentProvider {
    PayPal,
}

impl FromSql<'_> for PaymentProvider {
    fn from_sql(_ty: &Type, raw: &[u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        let s = String::from_utf8_lossy(raw);
        match s.as_ref() {
            "paypal" => Ok(PaymentProvider::PayPal),
            _ => Err("invalid payment provider".into()),
        }
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "payment_provider"
    }
}

impl ToSql for PaymentProvider {
    fn to_sql(
        &self,
        _ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        let s = match self {
            PaymentProvider::PayPal => "paypal",
        };
        out.extend_from_slice(s.as_bytes());
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "payment_provider"
    }

    fn to_sql_checked(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        self.to_sql(ty, out)
    }
}

pub async fn store_payment_order(
    pool: &Pool,
    user_id: i32,
    order: &PayPalOrder,
    req: &CreatePaymentRequest,
) -> Result<(), PaymentError> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;

    // Store payment record
    transaction
        .execute(
            "INSERT INTO payments (
            user_id, provider, provider_payment_id,
            amount_cents, currency, status,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, 'pending', NOW(), NOW())",
            &[
                &user_id,
                &PaymentProvider::PayPal,
                &order.id,
                &(req.amount_cents as i64),
                &req.currency,
            ],
        )
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|e| PaymentError::Transaction(format!("Failed to commit transaction: {}", e)))?;

    Ok(())
}

pub enum PaymentCredit {
    Credited,
    AlreadySettled,
    NotFound,
}

pub async fn paypal_payment_owner(
    pool: &Pool,
    provider_payment_id: &str,
) -> Result<Option<i32>, PaymentError> {
    let client = pool
        .get()
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;
    let row = client
        .query_opt(
            "SELECT user_id FROM payments
             WHERE provider = 'paypal' AND provider_payment_id = $1",
            &[&provider_payment_id],
        )
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;
    Ok(row.map(|row| row.get("user_id")))
}

/// Credit the stored order amount once. Capture webhooks identify a capture id;
/// pass the checkout order id as well. Repeated calls do not credit again.
pub async fn credit_captured_paypal_payment(
    pool: &Pool,
    provider_payment_ids: &[String],
) -> Result<PaymentCredit, PaymentError> {
    if provider_payment_ids.is_empty() {
        return Ok(PaymentCredit::NotFound);
    }
    let mut client = pool
        .get()
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;

    let row = transaction
        .query_opt(
            "UPDATE payments
             SET status = 'succeeded',
                 completed_at = NOW(),
                 updated_at = NOW()
             WHERE provider = 'paypal'
               AND provider_payment_id = ANY($1)
               AND status = 'pending'
             RETURNING id, user_id, amount_cents, currency",
            &[&provider_payment_ids],
        )
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;

    if let Some(row) = row {
        let payment_id: i32 = row.get("id");
        let user_id: i32 = row.get("user_id");
        let amount_cents: i64 = row.get("amount_cents");
        let currency: String = row.get("currency");
        let reference = format!("payment:{payment_id}");
        transaction
            .execute(
                "INSERT INTO balance_transactions (
                    user_id, amount_cents, currency, transaction_type, reference_id, created_at
                 ) VALUES ($1, $2, $3, 'payment', $4, NOW())",
                &[&user_id, &amount_cents, &currency, &reference],
            )
            .await
            .map_err(|e| PaymentError::Database(e.to_string()))?;
        transaction
            .execute(
                "INSERT INTO user_balances (user_id, balance_cents, created_at, updated_at)
                 VALUES ($1, $2, NOW(), NOW())
                 ON CONFLICT (user_id) DO UPDATE
                 SET balance_cents = user_balances.balance_cents + EXCLUDED.balance_cents,
                     updated_at = NOW()",
                &[&user_id, &amount_cents],
            )
            .await
            .map_err(|e| PaymentError::Database(e.to_string()))?;
        transaction
            .commit()
            .await
            .map_err(|e| PaymentError::Transaction(e.to_string()))?;
        return Ok(PaymentCredit::Credited);
    }

    let settled = transaction
        .query_opt(
            "SELECT 1 FROM payments
             WHERE provider = 'paypal'
               AND provider_payment_id = ANY($1)
               AND status = 'succeeded'",
            &[&provider_payment_ids],
        )
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;
    transaction
        .commit()
        .await
        .map_err(|e| PaymentError::Transaction(e.to_string()))?;
    if settled.is_some() {
        Ok(PaymentCredit::AlreadySettled)
    } else {
        Ok(PaymentCredit::NotFound)
    }
}

pub async fn mark_paypal_payment_failed(
    pool: &Pool,
    provider_payment_ids: &[String],
) -> Result<u64, PaymentError> {
    if provider_payment_ids.is_empty() {
        return Ok(0);
    }
    let client = pool
        .get()
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;
    client
        .execute(
            "UPDATE payments
             SET status = 'failed', updated_at = NOW()
             WHERE provider = 'paypal'
               AND provider_payment_id = ANY($1)
               AND status = 'pending'",
            &[&provider_payment_ids],
        )
        .await
        .map(|count| count)
        .map_err(|e| PaymentError::Database(e.to_string()))
}

pub async fn get_balance(
    pool: &Pool,
    user_id: i32,
) -> Result<BalanceResponse, Box<dyn std::error::Error>> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;

    let transaction = client
        .transaction()
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;

    // Try to get existing balance
    let result = transaction
        .query_opt(
            "SELECT balance_cents, premium_expires_at 
             FROM user_balances 
             WHERE user_id = $1",
            &[&user_id],
        )
        .await?;

    match result {
        Some(row) => Ok(BalanceResponse {
            balance_cents: row.get::<_, i64>("balance_cents") as u64,
            premium_expires_at: row
                .get::<_, Option<chrono::DateTime<chrono::Utc>>>("premium_expires_at")
                .map(|dt| dt.to_rfc3339()),
        }),
        None => {
            // Create default balance record for new user
            transaction
                .execute(
                    "INSERT INTO user_balances (user_id, balance_cents, created_at, updated_at)
                 VALUES ($1, 0, NOW(), NOW())",
                    &[&user_id],
                )
                .await?;

            Ok(BalanceResponse {
                balance_cents: 0,
                premium_expires_at: None,
            })
        }
    }
}

// Helper functions for PayPal Subscriptions

fn parse_iso8601_to_utc(datetime_str: Option<&str>) -> Result<Option<DateTime<Utc>>, PaymentError> {
    match datetime_str {
        Some(s) => {
            if s.is_empty() {
                // Handle empty strings explicitly if they can occur
                return Ok(None);
            }
            chrono::DateTime::parse_from_rfc3339(s)
                .map(|dt| Some(dt.with_timezone(&Utc)))
                .map_err(|e| {
                    PaymentError::Transaction(format!("Failed to parse date string '{}': {}", s, e))
                })
        }
        None => Ok(None),
    }
}

fn parse_money_to_cents(money_str: Option<&str>) -> Result<Option<i64>, PaymentError> {
    match money_str {
        Some(s) => {
            if s.is_empty() {
                return Ok(None);
            }
            let val = s.parse::<f64>().map_err(|e| {
                PaymentError::Transaction(format!(
                    "Failed to parse money string '{}' to f64: {}",
                    s, e
                ))
            })?;
            Ok(Some((val * 100.0).round() as i64))
        }
        None => Ok(None),
    }
}

fn map_paypal_status_to_user_status(paypal_status: &str) -> &'static str {
    match paypal_status.to_uppercase().as_str() {
        "ACTIVE" => "active",
        "APPROVAL_PENDING" | "APPROVED" => "inactive", // APPROVED might mean user action needed
        "SUSPENDED" => "past_due",
        "CANCELLED" => "cancelled",
        "EXPIRED" => "inactive",
        _ => {
            warn!(
                "Unknown PayPal subscription status received: {}",
                paypal_status
            );
            "inactive" // Default to inactive for unknown statuses
        }
    }
}

fn map_webhook_event_to_user_status(event_type: &str, current_paypal_status: &str) -> &'static str {
    match event_type {
        paypal::PAYPAL_EVENT_SUB_ACTIVATED => "active",
        paypal::PAYPAL_EVENT_SUB_CANCELLED => "cancelled",
        paypal::PAYPAL_EVENT_SUB_EXPIRED => "inactive",
        paypal::PAYPAL_EVENT_SUB_SUSPENDED => "past_due",
        paypal::PAYPAL_EVENT_SUB_PAYMENT_FAILED => "past_due",
        paypal::PAYPAL_EVENT_PAYMENT_SALE_COMPLETED => {
            // If a payment completed for a subscription, it should ideally be active.
            // Rely on the resource's status if available and seems consistent.
            if current_paypal_status.to_uppercase() == "ACTIVE" {
                "active"
            } else {
                // If PayPal says ACTIVE after a sale, trust it. Otherwise, map current status.
                map_paypal_status_to_user_status(current_paypal_status)
            }
        }
        _ => {
            warn!("Unhandled webhook event type for status mapping: {}. Falling back to PayPal status.", event_type);
            map_paypal_status_to_user_status(current_paypal_status)
        }
    }
}

// PayPal Subscription Service Functions

pub async fn create_subscription_record(
    pool: &Pool,
    user_id: i32,
    paypal_sub_details: &dto::PayPalSubscriptionDto,
) -> Result<(), PaymentError> {
    info!(
        "Creating subscription record for user_id: {}, paypal_subscription_id: {}",
        user_id, paypal_sub_details.id
    );
    let mut client = pool
        .get()
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;

    let start_time_utc =
        parse_iso8601_to_utc(Some(&paypal_sub_details.start_time))?.ok_or_else(|| {
            PaymentError::Transaction("start_time is required for subscription".to_string())
        })?;

    let next_billing_time_utc = parse_iso8601_to_utc(
        paypal_sub_details
            .billing_info
            .as_ref()
            .and_then(|bi| bi.next_billing_time.as_deref()),
    )?;

    let last_payment_time_utc = parse_iso8601_to_utc(
        paypal_sub_details
            .billing_info
            .as_ref()
            .and_then(|bi| bi.last_payment.as_ref().and_then(|lp| lp.time.as_deref())),
    )?;

    let last_payment_amount_cents =
        parse_money_to_cents(paypal_sub_details.billing_info.as_ref().and_then(|bi| {
            bi.last_payment
                .as_ref()
                .and_then(|lp| lp.amount.as_ref().map(|m| m.value.as_str()))
        }))?;

    let last_payment_currency = paypal_sub_details
        .billing_info
        .as_ref()
        .and_then(|bi| bi.last_payment.as_ref())
        .and_then(|lp| lp.amount.as_ref())
        .map(|m| m.currency_code.clone());

    // Insert into paypal_subscriptions
    let insert_query = "
        INSERT INTO paypal_subscriptions (
            user_id, paypal_plan_id, paypal_subscription_id, status,
            start_time, next_billing_time, last_payment_time,
            last_payment_amount_cents, last_payment_currency,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        ON CONFLICT (paypal_subscription_id) DO UPDATE SET
            status = EXCLUDED.status,
            start_time = EXCLUDED.start_time,
            next_billing_time = EXCLUDED.next_billing_time,
            last_payment_time = EXCLUDED.last_payment_time,
            last_payment_amount_cents = EXCLUDED.last_payment_amount_cents,
            last_payment_currency = EXCLUDED.last_payment_currency,
            updated_at = NOW()
        RETURNING id";

    let sub_record = transaction
        .query_one(
            insert_query,
            &[
                &user_id,
                &paypal_sub_details.plan_id,
                &paypal_sub_details.id,
                &paypal_sub_details.status,
                &start_time_utc,
                &next_billing_time_utc,
                &last_payment_time_utc,
                &last_payment_amount_cents,
                &last_payment_currency,
            ],
        )
        .await
        .map_err(|e| {
            PaymentError::Database(format!(
                "Failed to insert/update paypal_subscription: {}",
                e
            ))
        })?;

    debug!(
        "Inserted/Updated paypal_subscription record with id: {}",
        sub_record.get::<_, i32>("id")
    );

    // Update users table
    let user_status = map_paypal_status_to_user_status(&paypal_sub_details.status);
    let paypal_customer_id = paypal_sub_details
        .subscriber
        .as_ref()
        .and_then(|s| s.payer_id.as_deref());

    transaction.execute(
        "UPDATE users SET subscription_status = $1, paypal_customer_id = $2, updated_at = NOW() WHERE userid = $3",
        &[&user_status, &paypal_customer_id, &user_id],
    ).await.map_err(|e| PaymentError::Database(format!("Failed to update users table for subscription: {}", e)))?;

    transaction.commit().await.map_err(|e| {
        PaymentError::Transaction(format!("Failed to commit subscription creation: {}", e))
    })?;
    info!(
        "Successfully created/updated subscription record for user_id: {}",
        user_id
    );
    Ok(())
}

pub async fn get_subscription_owner(
    pool: &Pool,
    paypal_subscription_id: &str,
) -> Result<Option<i32>, PaymentError> {
    let client = pool
        .get()
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;

    let row = client
        .query_opt(
            "SELECT user_id FROM paypal_subscriptions WHERE paypal_subscription_id = $1",
            &[&paypal_subscription_id],
        )
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;

    match row {
        Some(r) => Ok(Some(r.get("user_id"))),
        None => Ok(None),
    }
}

pub async fn update_subscription_from_webhook(
    pool: &Pool,
    webhook_resource: &dto::SubscriptionWebhookEventResource,
) -> Result<(), PaymentError> {
    info!(
        "Processing webhook event: {}, summary: {}, for subscription_id: {}",
        webhook_resource.event_type, webhook_resource.summary, webhook_resource.resource.id
    );

    let mut client = pool
        .get()
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;

    let paypal_subscription_id = &webhook_resource.resource.id;

    // Fetch user_id from paypal_subscriptions table
    let subscription_row = transaction
        .query_opt(
            "SELECT user_id FROM paypal_subscriptions WHERE paypal_subscription_id = $1",
            &[&paypal_subscription_id],
        )
        .await
        .map_err(|e| {
            PaymentError::Database(format!(
                "Failed to query for subscription {}: {}",
                paypal_subscription_id, e
            ))
        })?;

    let user_id: i32 = match subscription_row {
        Some(row) => row.get("user_id"),
        None => {
            warn!(
                "Received webhook for unknown subscription_id: {}. Ignoring.",
                paypal_subscription_id
            );
            // It's possible PayPal sends webhooks for subscriptions not fully initiated or cleaned up.
            // Or if it's an old one. So, log and return Ok.
            return Ok(());
        }
    };

    // Update paypal_subscriptions table
    let new_status = &webhook_resource.resource.status;
    let next_billing_time_utc = parse_iso8601_to_utc(
        webhook_resource
            .resource
            .billing_info
            .as_ref()
            .and_then(|bi| bi.next_billing_time.as_deref()),
    )?;
    let last_payment_time_utc = parse_iso8601_to_utc(
        webhook_resource
            .resource
            .billing_info
            .as_ref()
            .and_then(|bi| bi.last_payment.as_ref().and_then(|lp| lp.time.as_deref())),
    )?;
    let last_payment_amount_cents = parse_money_to_cents(
        webhook_resource
            .resource
            .billing_info
            .as_ref()
            .and_then(|bi| {
                bi.last_payment
                    .as_ref()
                    .and_then(|lp| lp.amount.as_ref().map(|m| m.value.as_str()))
            }),
    )?;
    let last_payment_currency = webhook_resource
        .resource
        .billing_info
        .as_ref()
        .and_then(|bi| bi.last_payment.as_ref())
        .and_then(|lp| lp.amount.as_ref())
        .map(|m| m.currency_code.clone());

    let mut cancelled_at_utc: Option<DateTime<Utc>> = None;
    let mut cancel_reason: Option<String> = None;

    if webhook_resource.event_type == paypal::PAYPAL_EVENT_SUB_CANCELLED {
        cancelled_at_utc = Some(Utc::now()); // Or parse from event if available and preferred
                                             // Use summary as cancel reason, or a part of it.
                                             // Be mindful of length if storing in DB.
        cancel_reason = Some(webhook_resource.summary.chars().take(255).collect());
        // Example: truncate
    }

    // If status is CANCELLED from resource, also set cancelled_at
    if new_status.to_uppercase() == "CANCELLED" && cancelled_at_utc.is_none() {
        cancelled_at_utc = Some(Utc::now());
        if cancel_reason.is_none() {
            cancel_reason = Some(format!(
                "Cancelled via webhook event: {}",
                webhook_resource.event_type
            ));
        }
    }

    transaction
        .execute(
            "UPDATE paypal_subscriptions SET
            status = $1,
            next_billing_time = $2,
            last_payment_time = $3,
            last_payment_amount_cents = $4,
            last_payment_currency = $5,
            cancelled_at = COALESCE($6, cancelled_at), 
            cancel_reason = COALESCE($7, cancel_reason),
            updated_at = NOW()
        WHERE paypal_subscription_id = $8",
            &[
                &new_status,
                &next_billing_time_utc,
                &last_payment_time_utc,
                &last_payment_amount_cents,
                &last_payment_currency,
                &cancelled_at_utc,
                &cancel_reason,
                &paypal_subscription_id,
            ],
        )
        .await
        .map_err(|e| {
            PaymentError::Database(format!(
                "Failed to update paypal_subscriptions for webhook: {}",
                e
            ))
        })?;

    // Update users table
    let user_status = map_webhook_event_to_user_status(&webhook_resource.event_type, new_status);
    transaction
        .execute(
            "UPDATE users SET subscription_status = $1, updated_at = NOW() WHERE userid = $2",
            &[&user_status, &user_id],
        )
        .await
        .map_err(|e| {
            PaymentError::Database(format!("Failed to update users table for webhook: {}", e))
        })?;

    transaction.commit().await.map_err(|e| {
        PaymentError::Transaction(format!("Failed to commit webhook update: {}", e))
    })?;
    info!(
        "Successfully processed webhook for subscription_id: {}",
        paypal_subscription_id
    );
    Ok(())
}

pub async fn process_subscription_cancellation(
    pool: &Pool,
    paypal_subscription_id: &str,
    reason: Option<&str>,
) -> Result<(), PaymentError> {
    info!(
        "Processing manual cancellation for subscription_id: {} with reason: {:?}",
        paypal_subscription_id, reason
    );
    let mut client = pool
        .get()
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| PaymentError::Database(e.to_string()))?;

    // Fetch user_id
    let subscription_row = transaction
        .query_opt(
            "SELECT user_id FROM paypal_subscriptions WHERE paypal_subscription_id = $1",
            &[&paypal_subscription_id],
        )
        .await
        .map_err(|e| {
            PaymentError::Database(format!(
                "Failed to query for subscription {}: {}",
                paypal_subscription_id, e
            ))
        })?;

    let user_id: i32 = match subscription_row {
        Some(row) => row.get("user_id"),
        None => {
            error!(
                "Attempted to process cancellation for unknown subscription_id: {}",
                paypal_subscription_id
            );
            return Err(PaymentError::NotFound(format!(
                "Subscription {} not found for cancellation.",
                paypal_subscription_id
            )));
        }
    };

    // Update paypal_subscriptions table
    let new_status = "CANCELLED"; // Explicitly set to CANCELLED
    let cancelled_at_utc = Utc::now();

    let updated_rows = transaction
        .execute(
            "UPDATE paypal_subscriptions SET
            status = $1,
            cancelled_at = $2,
            cancel_reason = $3,
            updated_at = NOW()
        WHERE paypal_subscription_id = $4",
            &[
                &new_status,
                &cancelled_at_utc,
                &reason,
                &paypal_subscription_id,
            ],
        )
        .await
        .map_err(|e| {
            PaymentError::Database(format!(
                "Failed to update paypal_subscriptions for cancellation: {}",
                e
            ))
        })?;

    if updated_rows == 0 {
        // This case should ideally be caught by the query_opt above, but as a safeguard:
        error!(
            "Subscription {} not found during update for cancellation, though user_id was fetched.",
            paypal_subscription_id
        );
        return Err(PaymentError::NotFound(format!(
            "Subscription {} disappeared during cancellation.",
            paypal_subscription_id
        )));
    }

    // Update users table
    let user_status = "cancelled"; // Directly set to cancelled
    transaction
        .execute(
            "UPDATE users SET subscription_status = $1, updated_at = NOW() WHERE userid = $2",
            &[&user_status, &user_id],
        )
        .await
        .map_err(|e| {
            PaymentError::Database(format!(
                "Failed to update users table for cancellation: {}",
                e
            ))
        })?;

    transaction
        .commit()
        .await
        .map_err(|e| PaymentError::Transaction(format!("Failed to commit cancellation: {}", e)))?;
    info!(
        "Successfully processed cancellation for subscription_id: {}",
        paypal_subscription_id
    );
    Ok(())
}
