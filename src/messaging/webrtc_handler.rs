// Functional WebRTC signaling handler for peer-to-peer communication

use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::{AppError, auth::Claims};
use super::service::MessagingService;
use super::models::SignalType;


// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct SendSignalRequest {
    signal_type: SignalType,
    signal_data: String,
    to_user_id: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct SignalResponse {
    id: i64,
    signal_type: SignalType,
    signal_data: String,
    from_user_id: i32,
    from_username: String,
    to_user_id: i32,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SignalListResponse {
    signals: Vec<SignalResponse>,
    total: i64,
}


// Send WebRTC signal
pub async fn send_signal(
    claims: Claims,
    request: web::Json<SendSignalRequest>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    let req = request.into_inner();
    
    // Validate signal type manually
    match req.signal_type {
        SignalType::Offer | SignalType::Answer | SignalType::IceCandidate => {},
    }
    
    // Check if users are blocking each other
    if service.is_user_blocked(req.to_user_id, claims.sub).await? {
        return Err(AppError::Auth("Cannot send signal to blocked user".to_string()));
    }
    
    // Create signal
    let signal = service
        .send_webrtc_signal(
            claims.sub,
            req.to_user_id,
            req.signal_type,
            req.signal_data.clone(),
        )
        .await?;
    
    let response = SignalResponse {
        id: signal.signal_id,
        signal_type: signal.signal_type,
        signal_data: signal.signal_data.to_string(),
        from_user_id: signal.from_user_id,
        from_username: "User".to_string(), // TODO: Get from database
        to_user_id: signal.to_user_id,
        created_at: signal.created_at,
        expires_at: signal.expires_at,
    };
    
    Ok(HttpResponse::Created().json(response))
}

// Get pending signals for a user
pub async fn get_pending_signals(
    claims: Claims,
    path: web::Path<i32>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    
    // Users can only get their own signals
    if claims.sub != user_id {
        return Err(AppError::Auth("Access denied".to_string()));
    }
    
    let signals = service
        .get_pending_webrtc_signals(user_id)
        .await?;
    
    let response_signals: Vec<SignalResponse> = signals
        .into_iter()
        .map(|signal| SignalResponse {
            id: signal.signal_id,
            signal_type: signal.signal_type,
            signal_data: signal.signal_data.to_string(),
            from_user_id: signal.from_user_id,
            from_username: "User".to_string(), // TODO: Get from database
            to_user_id: signal.to_user_id,
            created_at: signal.created_at,
            expires_at: signal.expires_at,
        })
        .collect();
    
    let total = response_signals.len() as i64;
    let response = SignalListResponse {
        signals: response_signals,
        total,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

// Mark signal as processed
pub async fn mark_signal_processed(
    claims: Claims,
    path: web::Path<i64>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    let signal_id = path.into_inner();
    
    // Mark signal as processed
    service
        .mark_webrtc_signal_processed(signal_id, claims.sub)
        .await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Signal marked as processed",
        "signal_id": signal_id
    })))
}

// Get WebRTC configuration for client
pub async fn get_webrtc_config(
    claims: Claims,
) -> Result<HttpResponse, AppError> {
    let config = serde_json::json!({
        "ice_servers": [
            {
                "urls": std::env::var("STUN_SERVER_URL")
                    .unwrap_or_else(|_| "stun:lensisku.lojban.org:3478".to_string())
            },
            {
                "urls": std::env::var("TURN_SERVER_URL")
                    .unwrap_or_else(|_| "turn:lensisku.lojban.org:3478".to_string()),
                "username": std::env::var("TURN_USERNAME")
                    .unwrap_or_else(|_| "lensisku".to_string()),
                "credential": std::env::var("TURN_SECRET")
                    .unwrap_or_else(|_| "".to_string())
            }
        ],
        "user_id": claims.sub,
        "timestamp": Utc::now()
    });
    
    Ok(HttpResponse::Ok().json(config))
}

// Clean up expired signals (admin endpoint)
pub async fn cleanup_expired_signals(
    _claims: Claims,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    // TODO: Add admin permission check
    let cleaned_count = service.cleanup_expired_webrtc_signals().await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Expired signals cleaned up",
        "cleaned_count": cleaned_count
    })))
}

// Get active calls for a user
pub async fn get_active_calls(
    claims: Claims,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    let active_calls = service.get_active_webrtc_calls(claims.sub).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "active_calls": active_calls,
        "user_id": claims.sub
    })))
}
