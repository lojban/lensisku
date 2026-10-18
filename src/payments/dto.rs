use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreatePaymentRequest {
    #[validate(range(min = 50))] // Minimum 50 cents
    pub amount_cents: u64,
    #[validate(length(min = 3, max = 3))]
    pub currency: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreatePaymentResponse {
    pub payment_id: String,
    pub redirect_url: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BalanceResponse {
    pub balance_cents: u64,
    pub premium_expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub success: bool,
    pub message: String,
}

// Added based on task requirements:

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateSubscriptionRequest {
    pub plan_id: String,
    pub custom_id: Option<String>,
    pub start_time: Option<String>, // ISO 8601 format, e.g., "2024-12-01T00:00:00Z"
                                    // Consider adding application_context if needed
                                    // pub application_context: Option<PayPalApplicationContextDto>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PayPalSubscriberNameDto {
    pub given_name: Option<String>,
    pub surname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PayPalShippingAddressDto {
    pub address_line_1: Option<String>,
    pub admin_area_2: Option<String>, // City
    pub postal_code: Option<String>,
    pub country_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PayPalSubscriberDto {
    pub email_address: Option<String>,
    pub payer_id: Option<String>,
    pub name: Option<PayPalSubscriberNameDto>,
    pub shipping_address: Option<PayPalShippingAddressDto>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PayPalMoneyDto {
    pub currency_code: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PayPalCycleExecutionDto {
    pub tenure_type: String, // e.g., "REGULAR", "TRIAL"
    pub sequence: i32,
    pub cycles_completed: i32,
    pub cycles_remaining: Option<i32>,
    pub total_cycles: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PayPalLastPaymentDetailsDto {
    pub amount: Option<PayPalMoneyDto>,
    pub time: Option<String>, // ISO 8601 format
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PayPalSubscriptionBillingInfoDto {
    pub outstanding_balance: Option<PayPalMoneyDto>,
    pub cycle_executions: Option<Vec<PayPalCycleExecutionDto>>,
    pub last_payment: Option<PayPalLastPaymentDetailsDto>,
    pub next_billing_time: Option<String>, // ISO 8601 format
    pub failed_payments_count: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PayPalLink {
    pub href: String,
    pub rel: String,
    #[serde(rename = "method", skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PayPalSubscriptionDto {
    pub id: String, // PayPal's subscription ID
    pub plan_id: String,
    pub status: String, // e.g., "APPROVAL_PENDING", "ACTIVE", "CANCELLED"
    pub status_update_time: Option<String>, // ISO 8601 format
    pub start_time: String, // ISO 8601 format
    pub quantity: Option<String>,
    pub shipping_amount: Option<PayPalMoneyDto>,
    pub subscriber: Option<PayPalSubscriberDto>,
    pub billing_info: Option<PayPalSubscriptionBillingInfoDto>,
    pub create_time: Option<String>, // ISO 8601 format
    pub update_time: Option<String>, // ISO 8601 format
    pub custom_id: Option<String>,
    pub plan_overridden: Option<bool>,
    pub links: Vec<PayPalLink>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CancelSubscriptionRequestDto {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct PayPalWebhookEvent {
    pub event_type: String,
    pub resource: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct PayPalRelatedIds {
    pub order_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PayPalSupplementaryData {
    pub related_ids: Option<PayPalRelatedIds>,
}

/// `PAYMENT.CAPTURE.*` resource. Orders are stored under the checkout order id,
/// while `id` on this payload is the capture id.
#[derive(Debug, Deserialize)]
pub struct PayPalCaptureResource {
    pub id: String,
    pub custom_id: Option<String>,
    pub amount: PayPalMoneyDto,
    pub supplementary_data: Option<PayPalSupplementaryData>,
}

impl PayPalCaptureResource {
    pub fn payment_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        if let Some(order_id) = self
            .supplementary_data
            .as_ref()
            .and_then(|data| data.related_ids.as_ref())
            .and_then(|ids| ids.order_id.clone())
            .filter(|id| !id.is_empty())
        {
            ids.push(order_id);
        }
        if !self.id.is_empty() && !ids.iter().any(|id| id == &self.id) {
            ids.push(self.id.clone());
        }
        ids
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SubscriptionWebhookEventResource {
    pub id: String, // Subscription ID
    pub create_time: Option<String>,
    pub resource_type: String, // e.g., "subscription"
    pub event_type: String,    // e.g., "BILLING.SUBSCRIPTION.ACTIVATED"
    pub summary: String,
    pub resource: PayPalSubscriptionDto, // The actual subscription object
    pub links: Option<Vec<PayPalLink>>,
}

#[cfg(test)]
mod tests {
    use super::{PayPalCaptureResource, PayPalMoneyDto, PayPalRelatedIds, PayPalSupplementaryData};

    #[test]
    fn capture_ids_prefer_checkout_order_id() {
        let resource = PayPalCaptureResource {
            id: "capture".into(),
            custom_id: Some("42".into()),
            amount: PayPalMoneyDto {
                currency_code: "USD".into(),
                value: "1.99".into(),
            },
            supplementary_data: Some(PayPalSupplementaryData {
                related_ids: Some(PayPalRelatedIds {
                    order_id: Some("order".into()),
                }),
            }),
        };
        assert_eq!(
            resource.payment_ids(),
            vec!["order".to_string(), "capture".to_string()]
        );
    }
}

#[cfg(test)]
mod cents_tests {
    use super::super::service::cents_from_decimal;

    #[test]
    fn parses_paypal_amounts_as_cents() {
        assert_eq!(cents_from_decimal("1.99"), Some(199));
        assert_eq!(cents_from_decimal("10"), Some(1000));
        assert_eq!(cents_from_decimal("0.5"), Some(50));
        assert_eq!(cents_from_decimal("1.999"), None);
        assert_eq!(cents_from_decimal("-1.00"), None);
    }
}
