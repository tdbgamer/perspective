use perspective_api::*;
use perspective_ffi::{Pool, Table};
use wasm_bindgen::prelude::*;

use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;

use prost::Message;
use protos::{request::ClientReq, *};
use tokio::sync::{oneshot::Receiver, Mutex};
use wasm_bindgen::{prelude::*, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::WebSocket;

mod console_tracing;
pub mod protos {
    include!(concat!(env!("OUT_DIR"), "/perspective.proto.rs"));
}

// WASI reactor initialization function.
// This is because we're not running a `main` entrypoint that
// results in an exit code, we're initializing a library such that
// we can consume its functions from JavaScript.
#[no_mangle]
extern "C" fn _initialize() {
    console_tracing::set_global_logging();
}

pub enum DeferredBytes {
    Ready(Vec<u8>),
    WaitFor(tokio::sync::oneshot::Sender<Vec<u8>>),
}

#[wasm_bindgen]
pub struct WasmWebSocketTransport {
    ws: Arc<WebSocket>,
    _cb: Closure<dyn FnMut(web_sys::MessageEvent)>,
    // TODO: Make fixed length buffers rather than Vecs
    recv_buffer: Arc<RefCell<HashMap<Id, DeferredBytes>>>,
}

#[wasm_bindgen]
impl WasmWebSocketTransport {
    #[wasm_bindgen(constructor)]
    pub async fn js_constructor(url: &str) -> JsTransport {
        JsTransport(Box::new(WasmWebSocketTransport::new(url).await.unwrap()))
    }

    pub async fn new(url: &str) -> Result<WasmWebSocketTransport, JsValue> {
        let ws = WebSocket::new(url)?;

        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let recv_buffer = Arc::new(RefCell::new(HashMap::<Id, DeferredBytes>::new()));
        let recv_buffer_clone = recv_buffer.clone();

        let onmessage_callback = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            let recv_buffer_clone = recv_buffer_clone.clone();
            spawn_local(async move {
                tracing::debug!("Beginning of onmessage");
                tracing::debug!("Type of message is {:?}", e.data());
                match e.data().dyn_into::<js_sys::ArrayBuffer>() {
                    Ok(ab) => {
                        tracing::debug!("Top of match");
                        let bytes = js_sys::Uint8Array::new(&ab).to_vec();
                        tracing::debug!("Got bytes: {:?}", bytes);
                        let env = MultiplexEnvelope::decode(bytes.as_slice()).unwrap();
                        tracing::debug!("Got Envelope: {:?}", env);
                        let mut recv_buffer = (*recv_buffer_clone).borrow_mut();

                        tracing::debug!("Received msg for id: {:?}", env.id);

                        match recv_buffer.remove(&env.id) {
                            Some(DeferredBytes::Ready(mut buf)) => {
                                buf.extend(&env.payload);
                                let _ = recv_buffer.insert(env.id, DeferredBytes::Ready(buf));
                            }
                            Some(DeferredBytes::WaitFor(sender)) => {
                                drop(recv_buffer);
                                sender.send(env.payload).unwrap();
                            }
                            None => {
                                recv_buffer.insert(env.id, DeferredBytes::Ready(env.payload));
                            }
                        };
                    }
                    Err(e) => {
                        tracing::error!("Received non-arraybuffer message: {:?}", e);
                    }
                }
            });
        }) as Box<dyn FnMut(_)>);

        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));

        // Ensure the WebSocket opens before returning
        let (onopen_tx, onopen_rx) = tokio::sync::oneshot::channel();
        let onopen_callback = Closure::once(move || {
            onopen_tx.send(()).unwrap();
        });
        // ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));

        // onopen_rx.await.unwrap();

        ws.set_onopen(None);
        drop(onopen_callback);

        Ok(WasmWebSocketTransport {
            ws: Arc::new(ws),
            // Only Drop the callback when the Transport is dropped
            _cb: onmessage_callback,
            recv_buffer,
        })
    }
}

#[async_trait(?Send)]
impl Transport for WasmWebSocketTransport {
    async fn send(&self, id: Id, msg: Vec<u8>) {
        // TODO: Don't mix up protobuf requirements in the transport maybe?
        //       Though technically the client layer can use whatever format it wants so maybe that's fine
        let env = MultiplexEnvelope { id, payload: msg };
        let mut bytes = Vec::new();
        env.encode(&mut bytes).unwrap();
        self.ws.send_with_u8_array(&bytes).unwrap();
    }
    async fn recv(&self, id: Id) -> Vec<u8> {
        let mut recv_buffer = (*self.recv_buffer).borrow_mut();
        if let Some(buf) = recv_buffer.remove(&id) {
            match buf {
                DeferredBytes::Ready(bytes) => bytes,
                DeferredBytes::WaitFor(_) => panic!("Unreachable"),
            }
        } else {
            let (tx, rx) = tokio::sync::oneshot::channel();
            recv_buffer.insert(id, DeferredBytes::WaitFor(tx));
            drop(recv_buffer);
            rx.await.unwrap()
        }
    }
}

#[wasm_bindgen]
pub struct RemotePerspectiveClient {
    transport: Box<dyn Transport>,
}
#[wasm_bindgen]
impl RemotePerspectiveClient {
    #[wasm_bindgen(constructor)]
    pub async fn js_constructor(transport: JsTransport) -> JsClient {
        JsClient(Arc::new(Self::new(transport.0)))
    }
    fn new(transport: Box<dyn Transport>) -> Self {
        RemotePerspectiveClient { transport }
    }
}

#[async_trait(?Send)]
impl PerspectiveClient for RemotePerspectiveClient {
    async fn make_table(self: Arc<Self>) -> perspective_api::Table {
        let req = Request {
            client_req: Some(ClientReq::MakeTableReq(MakeTableReq {})),
        };
        self.transport
            .send(0, prost::Message::encode_to_vec(&req))
            .await;
        let resp: Response =
            prost::Message::decode(self.transport.recv(0).await.as_slice()).unwrap();
        match resp.client_resp {
            Some(response::ClientResp::MakeTableResp(MakeTableResp { id })) => {
                perspective_api::Table::new(id, self)
            }
            _ => panic!("Unexpected response"),
        }
    }

    async fn table_size(&self, id: Id) -> usize {
        let req = Request {
            client_req: Some(request::ClientReq::TableSizeReq(TableSizeReq {})),
        };
        let bytes = prost::Message::encode_to_vec(&req);
        self.transport.send(id, bytes).await;
        let bytes = self.transport.recv(id).await;
        let b = prost::bytes::Bytes::from(bytes);
        let resp: Response = prost::Message::decode(b).unwrap();
        match resp.client_resp {
            Some(response::ClientResp::TableSizeResp(TableSizeResp { size, .. })) => size as usize,
            _ => panic!("Unexpected response"),
        }
    }
}

#[wasm_bindgen]
pub struct MemoryPerspectiveClient {
    tables: Mutex<HashMap<Id, perspective_ffi::Table>>,
}

#[wasm_bindgen]
impl MemoryPerspectiveClient {
    pub fn new() -> MemoryPerspectiveClient {
        MemoryPerspectiveClient {
            tables: Mutex::new(HashMap::new()),
        }
    }

    #[wasm_bindgen(constructor)]
    pub fn js_constructor() -> JsClient {
        JsClient(Arc::new(MemoryPerspectiveClient::new()))
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct JsClient(Arc<dyn PerspectiveClient>);

#[wasm_bindgen]
impl JsClient {
    #[wasm_bindgen(js_name = "makeTable")]
    pub async fn make_table_js(&self) -> JsTable {
        JsTable(self.0.clone().make_table().await)
    }
}

#[wasm_bindgen]
pub struct JsTransport(Box<dyn Transport>);

#[wasm_bindgen]
pub struct JsTable(perspective_api::Table);

#[wasm_bindgen]
impl JsTable {
    #[wasm_bindgen(js_name = "size")]
    pub async fn size(&self) -> usize {
        self.0.size().await
    }
}

#[async_trait(?Send)]
impl PerspectiveClient for MemoryPerspectiveClient {
    async fn table_size(&self, id: Id) -> usize {
        self.tables.lock().await.get(&id).unwrap().size()
    }
    async fn make_table(self: Arc<Self>) -> perspective_api::Table {
        let table = perspective_ffi::Table::new();
        self.tables.lock().await.insert(0, table);
        perspective_api::Table::new(0, self)
    }
}
