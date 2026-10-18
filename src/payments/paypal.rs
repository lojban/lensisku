use super::dto::{
    CancelSubscriptionRequestDto, CreateSubscriptionRequest, PayPalLink, PayPalSubscriptionDto,
};
use super::PaymentProvider;
use async_trait::async_trait;
use log::{debug, error, info};
use once_cell::sync::Lazy;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use utoipa::ToSchema;

pub fn format_cents(cents: u64) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}

const PAYPAL_API_URL: &str = "https://api.paypal.com";
const PAYPAL_SANDBOX_API_URL: &str = "https://api.sandbox.paypal.com";

// PayPal Webhook Event Types for Subscriptions
pub const PAYPAL_EVENT_SUB_ACTIVATED: &str = "BILLING.SUBSCRIPTION.ACTIVATED";
pub const PAYPAL_EVENT_SUB_CANCELLED: &str = "BILLING.SUBSCRIPTION.CANCELLED";
pub const PAYPAL_EVENT_SUB_EXPIRED: &str = "BILLING.SUBSCRIPTION.EXPIRED";
pub const PAYPAL_EVENT_SUB_SUSPENDED: &str = "BILLING.SUBSCRIPTION.SUSPENDED";
pub const PAYPAL_EVENT_SUB_PAYMENT_FAILED: &str = "BILLING.SUBSCRIPTION.PAYMENT.FAILED";
pub const PAYPAL_EVENT_PAYMENT_SALE_COMPLETED: &str = "PAYMENT.SALE.COMPLETED";

pub struct PayPalProvider {
    client: reqwest::Client,
    client_id: String,
    client_secret: String,
    sandbox_mode: bool,
}

#[derive(Debug, Serialize)]
struct CreateOrderRequest<'a> {
    intent: &'a str,
    purchase_units: Vec<PurchaseUnit>,
}

#[derive(Debug, Serialize)]
struct PurchaseUnit {
    amount: Amount,
    custom_id: String,
}

#[derive(Debug, Serialize)]
struct Amount {
    currency_code: String,
    value: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PayPalOrder {
    pub id: String,
    pub links: Vec<PayPalLink>,
}

// ELI5: We're creating a special safe (Mutex) to store our certificates, like a vault that only one person
// can open at a time. This prevents confusion when multiple parts of our program try to add or read
// certificates at the same time. The Lazy part means we only create this vault when we first need it.
static CERT_CACHE: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

impl PayPalProvider {
    pub fn shared() -> Result<&'static Self, Box<dyn std::error::Error + Send + Sync>> {
        static SHARED: OnceLock<PayPalProvider> = OnceLock::new();
        if let Some(provider) = SHARED.get() {
            return Ok(provider);
        }
        let provider = Self::new(
            std::env::var("PAYPAL_CLIENT_ID").unwrap_or_default(),
            std::env::var("PAYPAL_CLIENT_SECRET").unwrap_or_default(),
        )?;
        let _ = SHARED.set(provider);
        SHARED
            .get()
            .ok_or_else(|| "PayPal provider unavailable".into())
    }

    pub fn new(
        client_id: String,
        client_secret: String,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let sandbox_mode = std::env::var("PAYPAL_SANDBOX_MODE")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            client_id,
            client_secret,
            sandbox_mode,
        })
    }

    fn api_url(&self) -> &str {
        if self.sandbox_mode {
            PAYPAL_SANDBOX_API_URL
        } else {
            PAYPAL_API_URL
        }
    }

    async fn download_and_cache_cert(
        &self,
        url: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use openssl::x509::X509;

        {
            let cache = CERT_CACHE.lock().map_err(|e| e.to_string())?;
            if let Some(cert) = cache.get(url) {
                return Ok(cert.clone());
            }
        }

        if !url.starts_with("https://api.paypal.com/") {
            return Err("Invalid certificate URL".into());
        }

        // Download certificate
        let cert_pem = self.client.get(url).send().await?.text().await?;

        // Validate certificate
        let cert = X509::from_pem(cert_pem.as_bytes())?;

        // Verify certificate time validity
        let now = openssl::asn1::Asn1Time::days_from_now(0)?;
        let not_before = cert.not_before();
        let not_after = cert.not_after();

        use std::cmp::Ordering;

        // Compare against not_before (cert shouldn't be used before this time)
        if let Ok(ordering) = now.compare(not_before) {
            if ordering == Ordering::Less {
                return Err("Certificate is not yet valid".into());
            }
        }

        // Compare against not_after (cert shouldn't be used after this time)
        if let Ok(ordering) = now.compare(not_after) {
            if ordering == Ordering::Greater {
                return Err("Certificate is expired".into());
            }
        }

        {
            let mut cache = CERT_CACHE.lock().map_err(|e| e.to_string())?;
            cache.insert(url.to_string(), cert_pem.clone());
        }

        Ok(cert_pem)
    }

    async fn get_access_token(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        log::debug!("Getting PayPal access token");
        let mut headers = HeaderMap::new();
        let auth = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("{}:{}", self.client_id, self.client_secret),
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {}", auth))?,
        );

        let params = [("grant_type", "client_credentials")];
        let url = format!("{}/v1/oauth2/token", self.api_url());
        log::debug!("Making PayPal token request to: {}", url);
        log::debug!("Using client_id: {}", self.client_id);
        log::debug!("Sandbox mode: {}", self.sandbox_mode);

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                log::error!("PayPal token request failed: {}", e);
                Box::new(e) as Box<dyn std::error::Error + Send + Sync>
            })?;

        let status = response.status();
        let response_body = response.text().await.map_err(|e| {
            log::error!("Failed to read PayPal token response: {}", e);
            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
        })?;

        log::debug!("PayPal token response: {} - {}", status, response_body);

        if !status.is_success() {
            log::error!("PayPal token request failed with status: {}", status);
            let err_msg = format!(
                "PayPal token request failed: {} - {}",
                status, response_body
            );
            return Err(Box::from(err_msg) as Box<dyn std::error::Error + Send + Sync>);
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
        }

        let token: TokenResponse = serde_json::from_str(&response_body).map_err(|e| {
            log::error!(
                "Failed to parse PayPal token response: {} - {}",
                e,
                response_body
            );
            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
        })?;

        log::debug!("Successfully obtained PayPal access token");
        Ok(token.access_token)
    }

    pub async fn create_order(
        &self,
        amount_cents: u64,
        currency: &str,
        user_id: i32,
    ) -> Result<PayPalOrder, Box<dyn std::error::Error + Send + Sync>> {
        // Convert amount to dollars with 2 decimal places
        info!(
            "Creating PayPal order for user {}: {} {}",
            user_id,
            amount_cents / 100,
            currency
        );
        let access_token = self.get_access_token().await?;
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", access_token))?,
        );

        let amount = Amount {
            currency_code: currency.to_uppercase(),
            value: format_cents(amount_cents),
        };

        let purchase_unit = PurchaseUnit {
            amount,
            custom_id: user_id.to_string(),
        };

        let order_request = CreateOrderRequest {
            intent: "CAPTURE",
            purchase_units: vec![purchase_unit],
        };

        let url = format!("{}/v2/checkout/orders", self.api_url());
        log::debug!("Creating PayPal order at: {}", url);
        log::debug!("Order request payload: {:?}", order_request);

        let response = self
            .client
            .post(&url)
            .headers(headers.clone())
            .json(&order_request)
            .send()
            .await
            .map_err(|e| {
                log::error!("PayPal API request failed: {}", e);
                log::debug!("Request headers: {:?}", headers);
                Box::new(e) as Box<dyn std::error::Error + Send + Sync>
            })?;

        let status = response.status();
        let response_body = response.text().await.map_err(|e| {
            log::error!("Failed to read PayPal response: {}", e);
            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
        })?;

        log::debug!("PayPal API response: {} - {}", status, response_body);

        if !status.is_success() {
            #[derive(Debug, Deserialize)]
            struct PayPalError {
                name: String,
                message: String,
                details: Option<Vec<PayPalErrorDetail>>,
            }

            #[derive(Debug, Deserialize)]
            struct PayPalErrorDetail {
                field: String,
                issue: String,
                description: String,
            }

            let error_msg = match serde_json::from_str::<PayPalError>(&response_body) {
                Ok(err) => {
                    let details = err.details.map_or(String::new(), |d| {
                        let mut result = String::with_capacity(d.len() * 64);
                        for detail in d.iter() {
                            result.push_str("\n- ");
                            result.push_str(&detail.field);
                            result.push_str(": ");
                            result.push_str(&detail.issue);
                            result.push_str(" (");
                            result.push_str(&detail.description);
                            result.push(')');
                        }
                        result
                    });
                    format!("PayPal API error: {}: {}{}", err.name, err.message, details)
                }
                Err(_) => format!("PayPal API error: {} - {}", status, response_body),
            };

            log::error!("{}", error_msg);
            return Err(Box::from(error_msg) as Box<dyn std::error::Error + Send + Sync>);
        }

        let order: PayPalOrder = serde_json::from_str(&response_body).map_err(|e| {
            let err_msg = format!(
                "Failed to parse PayPal response: {} - Response: {}",
                e, response_body
            );
            log::error!("{}", err_msg);
            Box::from(err_msg) as Box<dyn std::error::Error + Send + Sync>
        })?;
        Ok(order)
    }

    pub async fn capture_order(
        &self,
        order_id: &str,
    ) -> Result<PayPalOrder, Box<dyn std::error::Error + Send + Sync>> {
        let access_token = self.get_access_token().await?;
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", access_token))?,
        );

        let url = format!("{}/v2/checkout/orders/{}/capture", self.api_url(), order_id);
        log::debug!("Capturing PayPal order at: {}", url);

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| {
                log::error!("PayPal capture request failed: {}", e);
                Box::new(e) as Box<dyn std::error::Error + Send + Sync>
            })?;

        let status = response.status();
        let response_body = response.text().await.map_err(|e| {
            log::error!("Failed to read PayPal capture response: {}", e);
            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
        })?;

        log::debug!("PayPal capture response: {} - {}", status, response_body);

        if !status.is_success() {
            #[derive(Debug, Deserialize)]
            struct PayPalError {
                name: String,
                message: String,
                details: Option<Vec<PayPalErrorDetail>>,
            }

            #[derive(Debug, Deserialize)]
            struct PayPalErrorDetail {
                field: String,
                issue: String,
                description: String,
            }

            let error_msg = match serde_json::from_str::<PayPalError>(&response_body) {
                Ok(err) => {
                    let details = err.details.map_or(String::new(), |d| {
                        let mut result = String::with_capacity(d.len() * 64);
                        for detail in d.iter() {
                            result.push_str("\n- ");
                            result.push_str(&detail.field);
                            result.push_str(": ");
                            result.push_str(&detail.issue);
                            result.push_str(" (");
                            result.push_str(&detail.description);
                            result.push(')');
                        }
                        result
                    });
                    format!(
                        "PayPal capture error: {}: {}{}",
                        err.name, err.message, details
                    )
                }
                Err(_) => format!("PayPal capture error: {} - {}", status, response_body),
            };

            log::error!("{}", error_msg);
            return Err(Box::from(error_msg) as Box<dyn std::error::Error + Send + Sync>);
        }

        let order: PayPalOrder = serde_json::from_str(&response_body).map_err(|e| {
            let err_msg = format!(
                "Failed to parse PayPal capture response: {} - Response: {}",
                e, response_body
            );
            log::error!("{}", err_msg);
            Box::from(err_msg) as Box<dyn std::error::Error + Send + Sync>
        })?;
        Ok(order)
    }

    pub async fn create_subscription(
        &self,
        request: &CreateSubscriptionRequest,
    ) -> Result<PayPalSubscriptionDto, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Creating PayPal subscription with plan_id: {}",
            request.plan_id
        );
        let access_token = self.get_access_token().await?;
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", access_token))?,
        );
        headers.insert(
            HeaderName::from_static("prefer"),
            HeaderValue::from_static("return=representation"),
        );

        let url = format!("{}/v1/billing/subscriptions", self.api_url());
        debug!("Creating PayPal subscription at: {}", url);
        debug!("Subscription request payload: {:?}", request);

        let response = self
            .client
            .post(&url)
            .headers(headers.clone())
            .json(request)
            .send()
            .await
            .map_err(|e| {
                error!("PayPal create subscription API request failed: {}", e);
                debug!("Request headers: {:?}", headers);
                format!("PayPal create subscription API request failed: {}", e)
            })?;

        let status = response.status();
        let response_body = response.text().await.map_err(|e| {
            error!("Failed to read PayPal create subscription response: {}", e);
            format!("Failed to read PayPal create subscription response: {}", e)
        })?;

        debug!(
            "PayPal create subscription API response: {} - {}",
            status, response_body
        );

        if !status.is_success() {
            // Re-use existing error parsing logic if applicable or adapt
            error!(
                "PayPal create subscription request failed with status: {} and body: {}",
                status, response_body
            );
            return Err(format!(
                "PayPal create subscription request failed: {} - {}",
                status, response_body
            )
            .into());
        }

        let subscription: PayPalSubscriptionDto =
            serde_json::from_str(&response_body).map_err(|e| {
                let err_msg = format!(
                    "Failed to parse PayPal create subscription response: {} - Response: {}",
                    e, response_body
                );
                error!("{}", err_msg);
                err_msg
            })?;
        Ok(subscription)
    }

    pub async fn get_subscription_details(
        &self,
        subscription_id: &str,
    ) -> Result<PayPalSubscriptionDto, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Getting PayPal subscription details for ID: {}",
            subscription_id
        );
        let access_token = self.get_access_token().await?;
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", access_token))?,
        );

        let url = format!(
            "{}/v1/billing/subscriptions/{}",
            self.api_url(),
            subscription_id
        );
        debug!("Getting PayPal subscription details from: {}", url);

        let response = self
            .client
            .get(&url)
            .headers(headers.clone())
            .send()
            .await
            .map_err(|e| {
                error!(
                    "PayPal get subscription details API request failed for ID {}: {}",
                    subscription_id, e
                );
                debug!("Request headers: {:?}", headers);
                format!(
                    "PayPal get subscription details API request failed for ID {}: {}",
                    subscription_id, e
                )
            })?;

        let status = response.status();
        let response_body = response.text().await.map_err(|e| {
            error!(
                "Failed to read PayPal get subscription details response for ID {}: {}",
                subscription_id, e
            );
            format!(
                "Failed to read PayPal get subscription details response for ID {}: {}",
                subscription_id, e
            )
        })?;

        debug!(
            "PayPal get subscription details API response for ID {}: {} - {}",
            subscription_id, status, response_body
        );

        if !status.is_success() {
            error!(
                "PayPal get subscription details request failed for ID {} with status: {} and body: {}",
                subscription_id, status, response_body
            );
            return Err(format!(
                "PayPal get subscription details request failed for ID {}: {} - {}",
                subscription_id, status, response_body
            )
            .into());
        }

        let subscription: PayPalSubscriptionDto =
            serde_json::from_str(&response_body).map_err(|e| {
                let err_msg = format!(
                    "Failed to parse PayPal get subscription details response for ID {}: {} - Response: {}",
                    subscription_id, e, response_body
                );
                error!("{}", err_msg);
                err_msg
            })?;
        Ok(subscription)
    }

    pub async fn cancel_subscription(
        &self,
        subscription_id: &str,
        reason: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Cancelling PayPal subscription ID: {} with reason: {}",
            subscription_id, reason
        );
        let access_token = self.get_access_token().await?;
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", access_token))?,
        );

        let request_body = CancelSubscriptionRequestDto {
            reason: reason.to_string(),
        };

        let url = format!(
            "{}/v1/billing/subscriptions/{}/cancel",
            self.api_url(),
            subscription_id
        );
        debug!("Cancelling PayPal subscription at: {}", url);
        debug!("Cancel subscription request payload: {:?}", request_body);

        let response = self
            .client
            .post(&url)
            .headers(headers.clone())
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                error!(
                    "PayPal cancel subscription API request failed for ID {}: {}",
                    subscription_id, e
                );
                debug!("Request headers: {:?}", headers);
                format!(
                    "PayPal cancel subscription API request failed for ID {}: {}",
                    subscription_id, e
                )
            })?;

        let status = response.status();
        // PayPal typically returns 204 No Content on successful cancellation.
        if status == reqwest::StatusCode::NO_CONTENT {
            info!(
                "Successfully cancelled PayPal subscription ID: {}",
                subscription_id
            );
            Ok(())
        } else {
            let response_body = response.text().await.unwrap_or_else(|e| {
                error!(
                    "Failed to read PayPal cancel subscription error response body for ID {}: {}",
                    subscription_id, e
                );
                format!("Failed to read error response body: {}", e)
            });
            error!(
                "PayPal cancel subscription request failed for ID {} with status: {} and body: {}",
                subscription_id, status, response_body
            );
            Err(format!(
                "PayPal cancel subscription request failed for ID {}: {} - {}",
                subscription_id, status, response_body
            )
            .into())
        }
    }
}

#[async_trait]
impl PaymentProvider for PayPalProvider {
    async fn verify_webhook_signature(
        &self,
        payload: &str,
        signature_b64: &str,
        transmission_id: &str,
        timestamp: &str,
        cert_url: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
        use crc32fast::hash;
        use openssl::sign::Verifier;
        use openssl::x509::X509;

        let webhook_id = std::env::var("PAYPAL_WEBHOOK_ID")
            .map_err(|_| "PAYPAL_WEBHOOK_ID environment variable not set")?;

        log::info!("\nPayPal Webhook Validation Debug:");
        log::info!("Raw payload length: {}", payload.len());
        log::info!(
            "Raw payload (first 100 chars): {}",
            &payload[..payload.len().min(100)]
        );

        log::info!("\nHeaders:");
        log::info!("Transmission ID: {}", transmission_id);
        log::info!("Timestamp: {}", timestamp);
        log::info!("Cert URL: {}", cert_url);
        log::info!(
            "Signature (first 50): {}",
            &signature_b64[..50.min(signature_b64.len())]
        );
        log::info!("Webhook ID: {}", webhook_id);

        // Calculate CRC32
        let crc = hash(payload.as_bytes());
        log::info!("\nCRC32 Calculation:");
        log::info!("Raw CRC (hex): {:x}", crc);
        log::info!("CRC (decimal): {}", crc);

        // Create validation message
        let message = format!("{transmission_id}|{timestamp}|{webhook_id}|{crc}");
        log::info!("\nValidation message: {}", message);

        // Download and validate certificate
        let cert_pem = self.download_and_cache_cert(cert_url).await?;
        log::info!("Certificate length: {}", cert_pem.len());

        // Decode signature
        let signature = BASE64.decode(signature_b64)?;
        log::info!("Decoded signature length: {}", signature.len());

        // Load certificate and get public key
        let cert = X509::from_pem(cert_pem.as_bytes())?;
        let public_key = cert.public_key()?;

        // Create and configure verifier
        let mut verifier = Verifier::new(openssl::hash::MessageDigest::sha256(), &public_key)?;
        verifier.update(message.as_bytes())?;

        // Verify signature
        let result = verifier.verify(&signature)?;
        log::info!("\nSignature verification result: {}", result);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::format_cents;

    #[test]
    fn formats_cents_without_binary_float_error() {
        assert_eq!(format_cents(0), "0.00");
        assert_eq!(format_cents(50), "0.50");
        assert_eq!(format_cents(100), "1.00");
        assert_eq!(format_cents(199), "1.99");
        assert_eq!(format_cents(10_000), "100.00");
    }
}
