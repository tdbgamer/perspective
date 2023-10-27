use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use axum::extract::ws::WebSocketUpgrade;
use axum::{
    extract::ws::{Message, WebSocket},
    headers,
    response::IntoResponse,
    routing::get,
};
use axum::{extract::ConnectInfo, Router, TypedHeader};
use futures::{SinkExt, StreamExt};
use tokio::sync::{Mutex, RwLock};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

pub mod protos {
    include!(concat!(env!("OUT_DIR"), "/perspective.proto.rs"));
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new().route("/ws", get(ws_handler)).layer(
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
    );

    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

struct WebsocketState {
    auto_inc: AtomicU32,
    tables: RwLock<HashMap<u32, perspective_ffi::Table>>,
}
unsafe impl Send for WebsocketState {}
unsafe impl Sync for WebsocketState {}

impl Default for WebsocketState {
    fn default() -> Self {
        WebsocketState {
            auto_inc: 1.into(),
            tables: Default::default(),
        }
    }
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    let (mut sender, mut receiver) = socket.split();

    let state = Arc::new(WebsocketState::default());

    let send_task = {
        tokio::spawn(async move {
            let state = state.clone();
            let sender = Mutex::new(sender);
            // TODO: This should all obviously use stuff in perspective-api, but just testing for now.
            receiver
                .for_each(|input| async {
                    let state = state.clone();
                    let envelope: protos::MultiplexEnvelope = match input.unwrap() {
                        Message::Binary(bin) => prost::Message::decode(bin.as_slice()).unwrap(),
                        _ => panic!("invalid msg type"),
                    };
                    let res: protos::Request =
                        prost::Message::decode(envelope.payload.as_slice()).unwrap();
                    tracing::debug!("Received Request: {:?}", res);
                    match res.client_req {
                        Some(protos::request::ClientReq::MakeTableReq(req)) => {
                            let table = perspective_ffi::Table::new();
                            let id = state.auto_inc.fetch_add(1, Ordering::SeqCst);
                            state.tables.write().await.insert(id, table);
                            let resp = protos::MultiplexEnvelope {
                                id: envelope.id,
                                payload: prost::Message::encode_to_vec(&protos::Response {
                                    client_resp: Some(protos::response::ClientResp::MakeTableResp(
                                        protos::MakeTableResp { id },
                                    )),
                                }),
                            };
                            let res = prost::Message::encode_to_vec(&resp);
                            sender
                                .lock()
                                .await
                                .send(Message::Binary(res))
                                .await
                                .unwrap();
                        }
                        Some(protos::request::ClientReq::TableSizeReq(req)) => {
                            if let Some(table) = state.tables.read().await.get(&envelope.id) {
                                let size = table.size() as u64;
                                let resp = protos::Response {
                                    client_resp: Some(protos::response::ClientResp::TableSizeResp(
                                        protos::TableSizeResp { size },
                                    )),
                                };
                                let envelope = protos::MultiplexEnvelope {
                                    id: envelope.id,
                                    payload: prost::Message::encode_to_vec(&resp),
                                };
                                let res = prost::Message::encode_to_vec(&envelope);

                                sender
                                    .lock()
                                    .await
                                    .send(Message::Binary(res))
                                    .await
                                    .unwrap();
                            }
                        }
                        _ => {}
                    }
                })
                .await;
        })
    };
    send_task.await.unwrap();
}
