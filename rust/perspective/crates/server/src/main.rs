// use std::{borrow::Cow, net::SocketAddr};

// use axum::extract::ws::WebSocketUpgrade;
// use axum::{
//     extract::ws::{Message, WebSocket},
//     headers,
//     response::IntoResponse,
//     routing::get,
// };
// use axum::{
//     extract::{ws::CloseFrame, ConnectInfo},
//     Router, TypedHeader,
// };
// use futures::SinkExt;
// use futures::StreamExt;
// use tower_http::trace::{DefaultMakeSpan, TraceLayer};
// use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

// pub mod protos {
//     include!(concat!(env!("OUT_DIR"), "/perspective.proto.rs"));
// }

// #[tokio::main]
// async fn main() {
//     tracing_subscriber::registry()
//         .with(
//             tracing_subscriber::EnvFilter::try_from_default_env()
//                 .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
//         )
//         .with(tracing_subscriber::fmt::layer())
//         .init();

//     let app = Router::new().route("/ws", get(ws_handler)).layer(
//         TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
//     );

//     let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();
//     tracing::debug!("listening on {}", addr);
//     axum::Server::bind(&addr)
//         .serve(app.into_make_service_with_connect_info::<SocketAddr>())
//         .await
//         .unwrap();
// }

// async fn ws_handler(
//     ws: WebSocketUpgrade,
//     user_agent: Option<TypedHeader<headers::UserAgent>>,
//     ConnectInfo(addr): ConnectInfo<SocketAddr>,
// ) -> impl IntoResponse {
//     let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
//         user_agent.to_string()
//     } else {
//         String::from("Unknown browser")
//     };
//     println!("`{user_agent}` at {addr} connected.");
//     ws.on_upgrade(move |socket| handle_socket(socket, addr))
// }

// async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
//     if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
//         println!("Pinged {who}...");
//     } else {
//         println!("Could not send ping {who}!");
//         return;
//     }

//     let (mut sender, mut _receiver) = socket.split();

//     let send_task = tokio::spawn(async move {
//         let n_msg = 20;
//         for i in 0..n_msg {
//             let foo = helloworld::HelloReply {
//                 message: format!("Hello from Tim!"),
//             };
//             let bar = helloworld::Messages {
//                 message: Some(helloworld::messages::Message::HelloReply(foo)),
//             };
//             let mut bytes = Vec::new();
//             prost::Message::encode(&bar, &mut bytes).unwrap();
//             // In case of any websocket error, we exit.
//             if sender.send(Message::Binary(bytes)).await.is_err() {
//                 return i;
//             }

//             tokio::time::sleep(std::time::Duration::from_millis(300)).await;
//         }

//         println!("Sending close to {who}...");
//         if let Err(e) = sender
//             .send(Message::Close(Some(CloseFrame {
//                 code: axum::extract::ws::close_code::NORMAL,
//                 reason: Cow::from("Goodbye"),
//             })))
//             .await
//         {
//             println!("Could not send Close due to {e}, probably it is ok?");
//         }
//         n_msg
//     });
//     send_task.await.unwrap();
// }

fn main() {
    println!("Mock");
}
