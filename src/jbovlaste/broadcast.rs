use actix_web_lab::sse::{self, Sse};
use actix_web_lab::util::InfallibleStream;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

pub struct Broadcaster {
    inner: Mutex<BroadcasterInner>,
}

#[derive(Debug, Clone)]
struct ClientInfo {
    sse_sender: mpsc::Sender<sse::Event>,
    cancel_sender: mpsc::Sender<bool>,
    is_cancelled: bool,
}

#[derive(Debug, Clone)]
struct BroadcasterInner {
    // Map client_id to ClientInfo
    clients: HashMap<String, ClientInfo>,
}

impl Broadcaster {
    // ELI5: We're creating a special messenger (Arc) that many parts of our program can use
    // to send messages to each other. It's like a shared bulletin board that everyone can
    // post messages on and read from at the same time, without getting mixed up.
    pub fn create() -> Arc<Self> {
        // Initialize BroadcasterInner manually since it no longer derives Default
        let inner = BroadcasterInner {
            clients: HashMap::new(),
        };
        Arc::new(Broadcaster {
            inner: Mutex::new(inner),
        })
    }

    #[allow(clippy::await_holding_lock)]
    pub async fn cancel_import(&self, client_id: &str) -> Result<(), String> {
        let mut inner = self.inner.lock();
        if let Some(client_info) = inner.clients.get_mut(client_id) {
            if !client_info.is_cancelled {
                client_info.is_cancelled = true;
                // Send cancellation signal
                client_info
                    .cancel_sender
                    .send(true)
                    .await
                    .map_err(|e| format!("Failed to send cancel signal: {}", e))?;
                Ok(())
            } else {
                Err("Import already cancelled".into())
            }
        } else {
            Err("Client ID not found".into())
        }
    }

    pub async fn list_active_imports(&self) -> Vec<String> {
        // Active imports are now just clients that haven't been cancelled
        self.inner
            .lock()
            .clients
            .iter()
            .filter(|(_, info)| !info.is_cancelled)
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub async fn new_client(
        &self,
    ) -> (
        String,
        Sse<InfallibleStream<ReceiverStream<sse::Event>>>,
        mpsc::Receiver<bool>,
    ) {
        let (sse_tx, sse_rx) = mpsc::channel(32);
        let (cancel_tx, cancel_rx) = mpsc::channel(1); // Channel for cancellation signal
        let client_id = Uuid::new_v4().to_string();

        let client_info = ClientInfo {
            sse_sender: sse_tx,
            cancel_sender: cancel_tx,
            is_cancelled: false,
        };

        self.inner
            .lock()
            .clients
            .insert(client_id.clone(), client_info);

        (
            client_id,
            Sse::from_infallible_receiver(sse_rx),
            cancel_rx, // Return the receiver for the service to listen on
        )
    }

    #[allow(clippy::await_holding_lock)]
    pub async fn broadcast(&self, client_id: &str, msg: &str) -> Result<(), String> {
        let inner = self.inner.lock();

        if let Some(client_info) = inner.clients.get(client_id) {
            if client_info
                .sse_sender
                .send(sse::Data::new(msg).into())
                .await
                .is_err()
            {
                // Error sending, client might have disconnected
                log::warn!("Failed to send SSE message to client {}", client_id);
                // Optionally remove the client here if send fails consistently
            }
        } else {
            log::warn!(
                "Attempted to broadcast to non-existent client {}",
                client_id
            );
            return Err("Failed to send to some clients".into());
        }

        Ok(())
    }

    pub async fn remove_client(&self, client_id: &str) {
        let mut inner = self.inner.lock();
        inner.clients.remove(client_id);
        log::info!("Removed client {}", client_id);
    }
}
