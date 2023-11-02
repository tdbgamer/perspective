use js_sys::Promise;
use perspective_api::*;
use perspective_ffi::{Pool, Table};
use wasm_bindgen::prelude::*;

use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;

use prost::Message;
use protos::{request::ClientReq, *};
use tokio::sync::{oneshot::Receiver, Mutex};
use wasm_bindgen::{prelude::*, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
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
#[derive(Default)]
pub struct PerspectiveTransport {
    outgoing: RefCell<Option<Box<dyn Fn(Vec<u8>)>>>,
    registered: RefCell<Vec<Box<dyn Fn(Vec<u8>)>>>,
}

#[wasm_bindgen]
impl PerspectiveTransport {
    #[wasm_bindgen(js_name = "make")]
    pub fn make() -> JsTransport {
        JsTransport(Arc::new(PerspectiveTransport::new()))
    }
}

impl PerspectiveTransport {
    pub fn new() -> PerspectiveTransport {
        PerspectiveTransport {
            ..Default::default()
        }
    }

    pub fn recv(&self, bytes: &[u8]) {
        for cb in self.registered.borrow().as_slice() {
            cb(bytes.to_vec());
        }
    }

    pub fn send(&self, bytes: &[u8]) {
        if let Some(cb) = self.outgoing.borrow().as_ref() {
            cb(bytes.to_vec());
        }
    }

    pub fn on_message(&self, cb: Box<dyn Fn(Vec<u8>)>) {
        *self.outgoing.borrow_mut() = Some(cb);
    }
}


#[wasm_bindgen]
pub struct RemotePerspectiveClient {
    transport: Arc<PerspectiveTransport>,
    recv_buffer: Arc<RefCell<HashMap<TableId, DeferredBytes>>>,
}
#[wasm_bindgen]
impl RemotePerspectiveClient {
    #[wasm_bindgen(js_name = "make")]
    pub async fn js_constructor(transport: &JsTransport) -> JsClient {
        JsClient(Arc::new(Self::new(transport.0.clone()).await))
    }

    async fn new(transport: Arc<PerspectiveTransport>) -> RemotePerspectiveClient {
        let recv_buffer_clone = Arc::new(RefCell::new(HashMap::<TableId, DeferredBytes>::new()));
        {
            let recv_buffer_clone = recv_buffer_clone.clone();
            transport
                .registered
                .borrow_mut()
                .push(Box::new(move |bytes| {
                    let recv_buffer_clone = recv_buffer_clone.clone();
                    spawn_local(async move {
                        let env = MultiplexEnvelope::decode(bytes.as_slice()).unwrap();
                        let mut recv_buffer = (*recv_buffer_clone).borrow_mut();

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
                    });
                }));
        }
        Self {
            transport,
            recv_buffer: recv_buffer_clone,
        }
    }
}

impl RemotePerspectiveClient {
    async fn recv(&self, id: TableId) -> Vec<u8> {
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

#[async_trait(?Send)]
impl PerspectiveClient for RemotePerspectiveClient {
    async fn make_table(self: Arc<Self>) -> perspective_api::Table {
        let req = Request {
            client_req: Some(ClientReq::MakeTableReq(MakeTableReq {})),
        };
        let envelope = MultiplexEnvelope {
            id: 0,
            payload: prost::Message::encode_to_vec(&req),
        };
        let msg = prost::Message::encode_to_vec(&envelope);
        self.transport.send(&msg);
        let data = self.recv(0).await;
        let resp: Response = prost::Message::decode(data.as_slice()).unwrap();
        match resp.client_resp {
            Some(response::ClientResp::MakeTableResp(MakeTableResp { id })) => {
                perspective_api::Table::new(id, self)
            }
            _ => panic!("Unexpected response"),
        }
    }

    async fn table_size(&self, id: TableId) -> usize {
        let req = Request {
            client_req: Some(request::ClientReq::TableSizeReq(TableSizeReq {})),
        };
        let bytes = prost::Message::encode_to_vec(&MultiplexEnvelope {
            id,
            payload: prost::Message::encode_to_vec(&req),
        });
        self.transport.send(&bytes);
        let bytes = self.recv(id).await;
        let resp: Response = prost::Message::decode(bytes.as_slice()).unwrap();
        match resp.client_resp {
            Some(response::ClientResp::TableSizeResp(TableSizeResp { size, .. })) => size as usize,
            _ => panic!("Unexpected response"),
        }
    }
}

#[wasm_bindgen]
pub struct MemoryPerspectiveClient {
    tables: Mutex<HashMap<TableId, perspective_ffi::Table>>,
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
pub struct JsTransport(Arc<PerspectiveTransport>);

#[wasm_bindgen]
impl JsTransport {
    #[wasm_bindgen(js_name = "onMessage")]
    pub fn on_message_js(&mut self, cb: js_sys::Function) {
        self.0.on_message(Box::new(move |bytes| {
            let this = JsValue::UNDEFINED;
            let bytes = js_sys::Uint8Array::from(bytes.as_slice());
            let _ = cb
                    .call1(&this, &bytes)
                    .unwrap()
                    // .dyn_into::<Promise>()
                    ;
            // Async call
        }));
    }

    // #[wasm_bindgen]
    // pub fn send(&self, bytes: &[u8]) {
    //     self.0.send(bytes);
    // }

    #[wasm_bindgen]
    pub fn recv(&self, bytes: &[u8]) {
        self.0.recv(bytes);
    }
}

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
    async fn table_size(&self, id: TableId) -> usize {
        self.tables.lock().await.get(&id).unwrap().size()
    }
    async fn make_table(self: Arc<Self>) -> perspective_api::Table {
        let table = perspective_ffi::Table::new();
        self.tables.lock().await.insert(0, table);
        perspective_api::Table::new(0, self)
    }
}
