// WebSocket service for real-time messaging

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use actix::{Actor, ActorContext, Addr, AsyncContext, Handler, Message, Running, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde_json::Value;

use super::dto::WebSocketMessage;
use crate::auth::decode_token;

static SESSION_ID: AtomicUsize = AtomicUsize::new(0);

const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: usize,
    pub addr: Addr<WsSession>,
    pub user_id: i32,
    pub username: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
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
pub struct BroadcastToThread {
    pub thread_id: i64,
    pub message: WebSocketMessage,
    pub exclude_user: Option<i32>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastToUsers {
    pub user_ids: Vec<i32>,
    pub message_json: String,
    pub exclude_user: Option<i32>,
}

pub struct WsSession {
    pub id: usize,
    pub user_id: i32,
    pub username: String,
    pub initial_thread_id: Option<i64>,
    pub lobby: Addr<ChatServer>,
    pub hb: Instant,
}

impl WsSession {
    pub fn new(
        user_id: i32,
        username: String,
        initial_thread_id: Option<i64>,
        lobby: Addr<ChatServer>,
    ) -> Self {
        Self {
            id: SESSION_ID.fetch_add(1, Ordering::SeqCst),
            user_id,
            username,
            initial_thread_id,
            lobby,
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

    fn handle_message(&mut self, message: Value, ctx: &mut ws::WebsocketContext<Self>) {
        let msg_type = message.get("type").and_then(Value::as_str);
        match msg_type {
            Some("join_thread") => {
                if let Some(thread_id) = message.get("thread_id").and_then(Value::as_i64) {
                    self.lobby.do_send(JoinThread {
                        user_id: self.user_id,
                        thread_id,
                    });
                }
            }
            Some("leave_thread") => {
                if let Some(thread_id) = message.get("thread_id").and_then(Value::as_i64) {
                    self.lobby.do_send(LeaveThread {
                        user_id: self.user_id,
                        thread_id,
                    });
                }
            }
            Some("typing") => {
                if let (Some(thread_id), Some(is_typing)) = (
                    message.get("thread_id").and_then(Value::as_i64),
                    message.get("is_typing").and_then(Value::as_bool),
                ) {
                    self.lobby.do_send(BroadcastToThread {
                        thread_id,
                        message: WebSocketMessage::Typing {
                            thread_id,
                            user_id: self.user_id,
                            username: self.username.clone(),
                            is_typing,
                        },
                        exclude_user: Some(self.user_id),
                    });
                }
            }
            Some("ping") => {
                ctx.text(r#"{"type":"pong"}"#);
            }
            _ => {}
        }
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();
        self.lobby.do_send(Connect {
            id: self.id,
            addr,
            user_id: self.user_id,
            username: self.username.clone(),
        });

        if let Some(thread_id) = self.initial_thread_id {
            self.lobby.do_send(JoinThread {
                user_id: self.user_id,
                thread_id,
            });
        }
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.lobby.do_send(Disconnect { id: self.id });
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
                if let Ok(message) = serde_json::from_str::<Value>(&text) {
                    self.handle_message(message, ctx);
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

impl Handler<WsMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

struct SessionInfo {
    user_id: i32,
    username: String,
    addr: Addr<WsSession>,
}

pub struct ChatServer {
    sessions: HashMap<usize, SessionInfo>,
    user_sessions: HashMap<i32, Vec<usize>>,
    thread_participants: HashMap<i64, Vec<i32>>,
}

impl ChatServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            user_sessions: HashMap::new(),
            thread_participants: HashMap::new(),
        }
    }

    fn broadcast_user_status(&self, user_id: i32, username: &str, status: &str) {
        let message = WebSocketMessage::UserStatus {
            user_id,
            username: username.to_string(),
            status: status.to_string(),
            thread_id: None,
        };

        if let Ok(json) = serde_json::to_string(&message) {
            for info in self.sessions.values() {
                info.addr.do_send(WsMessage(json.clone()));
            }
        }
    }
}

impl Actor for ChatServer {
    type Context = actix::Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) {
        self.user_sessions
            .entry(msg.user_id)
            .or_default()
            .push(msg.id);
        self.sessions.insert(
            msg.id,
            SessionInfo {
                user_id: msg.user_id,
                username: msg.username.clone(),
                addr: msg.addr,
            },
        );

        self.broadcast_user_status(msg.user_id, &msg.username, "online");
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) {
        if let Some(info) = self.sessions.remove(&msg.id) {
            if let Some(ids) = self.user_sessions.get_mut(&info.user_id) {
                ids.retain(|&id| id != msg.id);
                if ids.is_empty() {
                    self.user_sessions.remove(&info.user_id);
                    for participants in self.thread_participants.values_mut() {
                        participants.retain(|&uid| uid != info.user_id);
                    }
                    self.broadcast_user_status(info.user_id, &info.username, "offline");
                }
            }
        }
    }
}

impl Handler<JoinThread> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: JoinThread, _: &mut Self::Context) {
        let participants = self.thread_participants.entry(msg.thread_id).or_default();
        if !participants.contains(&msg.user_id) {
            participants.push(msg.user_id);
        }
    }
}

impl Handler<LeaveThread> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: LeaveThread, _: &mut Self::Context) {
        if let Some(participants) = self.thread_participants.get_mut(&msg.thread_id) {
            participants.retain(|&uid| uid != msg.user_id);
        }
    }
}

impl Handler<BroadcastToThread> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastToThread, _: &mut Self::Context) {
        if let Some(participants) = self.thread_participants.get(&msg.thread_id) {
            if let Ok(json) = serde_json::to_string(&msg.message) {
                for &user_id in participants {
                    if Some(user_id) == msg.exclude_user {
                        continue;
                    }
                    if let Some(ids) = self.user_sessions.get(&user_id) {
                        for id in ids {
                            if let Some(info) = self.sessions.get(id) {
                                info.addr.do_send(WsMessage(json.clone()));
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Handler<BroadcastToUsers> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastToUsers, _: &mut Self::Context) {
        for &user_id in &msg.user_ids {
            if Some(user_id) == msg.exclude_user {
                continue;
            }
            if let Some(ids) = self.user_sessions.get(&user_id) {
                for id in ids {
                    if let Some(info) = self.sessions.get(id) {
                        info.addr.do_send(WsMessage(msg.message_json.clone()));
                    }
                }
            }
        }
    }
}

/// Start a WebSocket session for a specific thread (legacy; automatically joins the thread).
pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<i64>,
    chat_server: web::Data<Addr<ChatServer>>,
) -> Result<HttpResponse, Error> {
    start_session(req, stream, chat_server, Some(path.into_inner())).await
}

/// Start a global WebSocket session; clients send `join_thread`/`leave_thread` messages.
pub async fn websocket_index_handler(
    req: HttpRequest,
    stream: web::Payload,
    chat_server: web::Data<Addr<ChatServer>>,
) -> Result<HttpResponse, Error> {
    start_session(req, stream, chat_server, None).await
}

async fn start_session(
    req: HttpRequest,
    stream: web::Payload,
    chat_server: web::Data<Addr<ChatServer>>,
    initial_thread_id: Option<i64>,
) -> Result<HttpResponse, Error> {
    let token = extract_token(&req);
    let claims = match token {
        Some(t) => match decode_token(&t) {
            Ok(c) => c,
            Err(_) => return Ok(HttpResponse::Unauthorized().finish()),
        },
        None => return Ok(HttpResponse::Unauthorized().finish()),
    };

    let session = WsSession::new(
        claims.sub,
        claims.username,
        initial_thread_id,
        chat_server.get_ref().clone(),
    );

    ws::start(session, &req, stream)
}

fn extract_token(req: &HttpRequest) -> Option<String> {
    req.query_string()
        .split('&')
        .find(|s| s.starts_with("access_token="))
        .and_then(|s| s.split_once('=').map(|x| x.1))
        .map(|s| {
            urlencoding::decode(s)
                .map(|cow| cow.to_string())
                .unwrap_or_else(|_| s.to_string())
        })
}
