mod channel;
mod client;
mod payload;
mod state;

use axum::{
    extract::{
        ws::{Message as WsMessage, WebSocket},
        WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use client::Client;
use futures::{Sink, SinkExt as _, StreamExt as _};
use state::State;

use crate::payload::{HugCommand, HugEvent};

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let state = State::default();

    let router = Router::new()
        .route("/websocket", get(socket_handler))
        .layer(Extension(state));

    Ok(router.into())
}

async fn socket_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<State>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn websocket(stream: WebSocket, state: State) {
    let (ws_sender, mut ws_receiver) = stream.split();
    let mut client = Client {
        state_ref: state.clone(),
        message_socket: None,
        ws_sender: Box::pin(ws_sender.with(|event: HugEvent| async move {
            Ok(WsMessage::Text(serde_json::to_string(&event).unwrap()))
        })),
    };
    tokio::spawn(async move {
        loop {
            tokio::select! {
                ws_message = ws_receiver.next() => {
                    match handle_ws_message(ws_message, &mut client).await {
                        Ok(Loop::Continue) => continue,
                        Ok(Loop::Break) => break,
                        Err(error) => {
                            tracing::error!("handle_ws_message error: {}", error);
                            break;
                        }
                    }
                },
                message = client.recv_message() => {
                    match client.handle_message(message).await {
                        Ok(()) => continue,
                        Err(error) => {
                            tracing::error!("handle_message error: {}", error);
                            break;
                        }
                    }
                },
            };
        }
    });
}

enum Loop {
    Continue,
    Break,
}

async fn handle_ws_message<T: Sink<HugEvent, Error = axum::Error> + Unpin>(
    ws_message: Option<Result<WsMessage, axum::Error>>,
    client: &mut Client<T>,
) -> anyhow::Result<Loop> {
    match ws_message {
        Some(Ok(WsMessage::Text(text))) => {
            let command: HugCommand = serde_json::from_str(&text).unwrap();
            match command {
                HugCommand::JoinRoom { key } => client.join_room(key.parse()?).await?,
                HugCommand::JoinRandom => client.join_random().await?,
                HugCommand::CreateRoom => client.create_room().await?,
                HugCommand::Leave => client.leave().await?,
                HugCommand::Push { payload } => client.push(payload).await?,
            }
            return Ok(Loop::Continue);
        }
        Some(Ok(WsMessage::Binary(bin))) => {
            tracing::error!("received binary message: {:?}", bin);
        }
        Some(Ok(WsMessage::Close(_))) => {
            tracing::info!("received close message");
            client.leave().await?;
            return Ok(Loop::Break);
        }
        Some(Ok(WsMessage::Ping(_))) | Some(Ok(WsMessage::Pong(_))) => {
            // handled by axum
            return Ok(Loop::Continue);
        }
        Some(Err(error)) => {
            tracing::error!("websocket error: {}", error);
            client.leave().await?;
        }
        None => {
            tracing::info!("websocket closed");
            client.leave().await?;
        }
    };
    Ok(Loop::Break)
}
