// Simple WebSocket handler for real-time messaging
// This implementation avoids the complex actor system for easier deployment

use actix_web::{web, HttpRequest, HttpResponse, Result, Error};
use actix_web_actors::ws;
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use super::service::MessagingService;

// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "chat")]
    Chat {
        id: i64,
        thread_id: i64,
        sender_id: i32,
        sender_name: String,
        content: String,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "typing")]
    Typing {
        thread_id: i64,
        user_id: i32,
        user_name: String,
        is_typing: bool,
    },
    #[serde(rename = "user_status")]
    UserStatus {
        user_id: i32,
        user_name: String,
        status: String, // "online", "offline", "away"
        thread_id: Option<i64>,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
    },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
}

// Session data for each connected user
#[derive(Debug)]
struct WsSession {
    user_id: i32,
    username: String,
    thread_id: i64,
    hb: Instant,
}

impl WsSession {
    fn new(user_id: i32, username: String, thread_id: i64) -> Self {
        Self {
            user_id,
            username,
            thread_id,
            hb: Instant::now(),
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        
        // Notify others that user is online
        let status_msg = WsMessage::UserStatus {
            user_id: self.user_id,
            user_name: self.username.clone(),
            status: "online".to_string(),
            thread_id: Some(self.thread_id),
        };
        
        if let Ok(msg_text) = serde_json::to_string(&status_msg) {
            ctx.text(msg_text);
        }
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // Notify others that user is offline
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    self.handle_message(ws_msg, ctx);
                } else {
                    let error_msg = WsMessage::Error {
                        message: "Invalid message format".to_string(),
                    };
                    if let Ok(msg_text) = serde_json::to_string(&error_msg) {
                        ctx.text(msg_text);
                    }
                }
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

impl WsSession {
    fn handle_message(&mut self, msg: WsMessage, ctx: &mut ws::WebsocketContext<Self>) {
        match msg {
            WsMessage::Chat { thread_id, content, .. } => {
                // For now, echo back the message
                // In a real implementation, this would save to database and broadcast
                let response = WsMessage::Chat {
                    id: 0, // Would be actual message ID from database
                    thread_id,
                    sender_id: self.user_id,
                    sender_name: self.username.clone(),
                    content,
                    timestamp: Utc::now(),
                };
                
                if let Ok(msg_text) = serde_json::to_string(&response) {
                    ctx.text(msg_text);
                }
            }
            WsMessage::Typing { thread_id, is_typing, .. } => {
                // Broadcast typing indicator
                let typing_msg = WsMessage::Typing {
                    thread_id,
                    user_id: self.user_id,
                    user_name: self.username.clone(),
                    is_typing,
                };
                
                if let Ok(msg_text) = serde_json::to_string(&typing_msg) {
                    ctx.text(msg_text);
                }
            }
            WsMessage::Ping => {
                let pong = WsMessage::Pong;
                if let Ok(msg_text) = serde_json::to_string(&pong) {
                    ctx.text(msg_text);
                }
            }
            _ => {}
        }
    }
}

// Constants for heartbeat
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

use std::time::{Duration, Instant};

// WebSocket handler functions
pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<i64>,
    _service: web::Data<MessagingService>,
) -> Result<HttpResponse, Error> {
    let thread_id = path.into_inner();
    
    // For now, create a simple WebSocket session without authentication
    // TODO: Implement proper JWT authentication
    let ws_session = WsSession::new(1, "user".to_string(), thread_id);
    
    ws::start(ws_session, &req, stream)
}

pub async fn websocket_index_handler(
    req: HttpRequest,
    stream: web::Payload,
    _service: web::Data<MessagingService>,
) -> Result<HttpResponse, Error> {
    // For now, create a simple WebSocket session without authentication
    // TODO: Implement proper JWT authentication
    let ws_session = WsSession::new(1, "user".to_string(), 0);
    
    ws::start(ws_session, &req, stream)
}
