// WebSocket service for real-time messaging
// This module will handle socket.io integration with HTTP long-polling fallback

use actix::{Actor, AsyncContext, Handler, Message, StreamHandler, Addr, Running, ActorContext};
use actix_web::{web, HttpRequest, HttpResponse, Error};
use actix_web_actors::ws;
use serde_json::Value;
use std::collections::HashMap;

use crate::auth::Claims;
use super::dto::WebSocketMessage;

// WebSocket message types
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Addr<WsSession>,
    pub user_id: i32,
    pub username: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub user_id: i32,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinThread {
    pub user_id: i32,
    pub thread_id: i64,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct LeaveThread {
    pub user_id: i32,
    pub thread_id: i64,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ThreadMessage {
    pub thread_id: i64,
    pub message: WebSocketMessage,
    pub exclude_user: Option<i32>,
}

// WebSocket session actor
pub struct WsSession {
    pub user_id: i32,
    pub username: String,
    pub addr: Addr<Self>,
    pub lobby: Addr<ChatServer>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.lobby.send(Connect {
            addr: ctx.address(),
            user_id: self.user_id,
            username: self.username.clone(),
        }).unwrap();
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.lobby.send(Disconnect {
            user_id: self.user_id,
        }).unwrap();
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // Handle incoming WebSocket messages
                if let Ok(message) = serde_json::from_str::<Value>(&text) {
                    self.handle_message(message, ctx);
                }
            }
            Ok(ws::Message::Binary(_)) => (),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

impl Handler<WsMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

// Chat server actor
pub struct ChatServer {
    sessions: HashMap<i32, Addr<WsSession>>,
    thread_participants: HashMap<i64, Vec<i32>>, // thread_id -> user_ids
}

impl ChatServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            thread_participants: HashMap::new(),
        }
    }
}

impl Actor for ChatServer {
    type Context = actix::Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) {
        self.sessions.insert(msg.user_id, msg.addr);
        
        // Notify others that user is online
        self.broadcast_user_status(msg.user_id, &msg.username, true);
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) {
        self.sessions.remove(&msg.user_id);
        
        // Remove from all threads
        for participants in self.thread_participants.values_mut() {
            participants.retain(|&id| id != msg.user_id);
        }
        
        // Notify others that user is offline
        self.broadcast_user_status(msg.user_id, "", false);
    }
}

impl Handler<JoinThread> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: JoinThread, _: &mut Self::Context) {
        self.thread_participants
            .entry(msg.thread_id)
            .or_insert_with(Vec::new)
            .push(msg.user_id);
    }
}

impl Handler<LeaveThread> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: LeaveThread, _: &mut Self::Context) {
        if let Some(participants) = self.thread_participants.get_mut(&msg.thread_id) {
            participants.retain(|&id| id != msg.user_id);
        }
    }
}

impl Handler<ThreadMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ThreadMessage, _: &mut Self::Context) {
        if let Some(participants) = self.thread_participants.get(&msg.thread_id) {
            let message_json = serde_json::to_string(&msg.message).unwrap();
            
            for &user_id in participants {
                if Some(user_id) != msg.exclude_user {
                    if let Some(addr) = self.sessions.get(&user_id) {
                        let _ = addr.do_send(WsMessage(message_json.clone()));
                    }
                }
            }
        }
    }
}

impl ChatServer {
    fn broadcast_user_status(&self, user_id: i32, username: &str, is_online: bool) {
        let message = if is_online {
            WebSocketMessage::UserOnline {
                user_id,
                username: username.to_string(),
            }
        } else {
            WebSocketMessage::UserOffline {
                user_id,
                username: username.to_string(),
            }
        };

        let message_json = serde_json::to_string(&message).unwrap();
        
        for addr in self.sessions.values() {
            let _ = addr.do_send(WsMessage(message_json.clone()));
        }
    }
}

// TODO: Implement HTTP long-polling fallback
// TODO: Implement proper socket.io integration
// TODO: Add message persistence and delivery guarantees
// TODO: Add typing indicators and read receipts
