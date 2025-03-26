use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;

type UserSockets = Arc<Mutex<HashMap<String, Vec<mpsc::UnboundedSender<Message>>>>>;

#[tokio::main]
async fn main() {
    let user_sockets = Arc::new(Mutex::new(HashMap::new()));

    let app = Router::new().route(
        "/ws",
        get(move |ws, query| websocket_handler(ws, query, Arc::clone(&user_sockets))),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!("WebSocket server running on ws://localhost:8000/ws?user_id=USER_ID");
    axum::serve(listener, app).await.unwrap();
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<HashMap<String, String>>,
    state: UserSockets,
) -> impl IntoResponse {
    if let Some(user_id) = params.get("user_id").cloned() {
        println!("user_id: {}", user_id);
        ws.on_upgrade(move |socket| handle_socket(socket, user_id, state))
    } else {
        "Missing user_id query parameter".into_response()
    }
}

async fn handle_socket(socket: WebSocket, user_id: String, state: UserSockets) {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let (mut sender, mut receiver) = socket.split();

    // Clone user_id upfront for use in async tasks and cleanup
    let user_id_task = user_id.clone();

    // Store the new connection
    {
        let mut user_sockets = state.lock().unwrap();
        user_sockets
            .entry(user_id.clone())
            .or_insert(Vec::new())
            .push(tx);
        println!("user_sockets: {:?}", user_sockets.get_key_value("19970330"));
    }

    // Task to receive messages from the WebSocket
    tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(t) => {
                    println!("Received text from {}: {}", user_id_task, t);
                }
                Message::Binary(d) => {
                    println!("Received binary from {}: {:?}", user_id_task, d);
                }
                Message::Ping(v) => {
                    println!("Ping from {}: {:?}", user_id_task, v);
                }
                Message::Pong(v) => {
                    println!("Pong from {}: {:?}", user_id_task, v);
                }
                Message::Close(c) => {
                    println!("Closed from {}: {:?}", user_id_task, c);
                    break;
                }
            }
        }
        // Cleanup when user disconnects
        let mut user_sockets = state.lock().unwrap();
        if let Some(sockets) = user_sockets.get_mut(&user_id_task) {
            sockets.retain(|s| !s.is_closed() && s.send(Message::Ping(vec![])).is_ok());
            if sockets.is_empty() {
                user_sockets.remove(&user_id_task);
            }
        }
    });

    // Task to send messages to the WebSocket
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break; // If sending fails, exit the loop
            }
        }
    });
}

#[allow(dead_code)]
async fn send_notification(user_id: &str, message: &str, state: &UserSockets) {
    let mut user_sockets = state.lock().unwrap();
    if let Some(sockets) = user_sockets.get_mut(user_id) {
        sockets.retain(|s| !s.is_closed());
        for sender in sockets.iter() {
            let _ = sender.send(Message::Text(message.to_string()));
        }
    }
}

#[allow(dead_code)]
async fn follow_user(follower_id: &str, followed_id: &str, state: &UserSockets) {
    let message = format!("User {} followed you!", follower_id);
    send_notification(followed_id, &message, state).await;
}
