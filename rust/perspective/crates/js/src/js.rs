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
    server_rx: RefCell<Option<Box<dyn Fn(Vec<u8>)>>>,
    client_rx: RefCell<Option<Box<dyn Fn(Vec<u8>)>>>,
    server_tx: RefCell<Option<Box<dyn Fn(Vec<u8>)>>>,
    client_tx: RefCell<Option<Box<dyn Fn(Vec<u8>)>>>,
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
}

impl Transport for PerspectiveTransport {
    fn server_rx(&self, msg: &[u8]) {
        if let Some(cb) = self.server_rx.borrow_mut().as_mut() {
            cb(msg.to_vec());
        } else {
            tracing::error!("Server rx not registered");
        }
    }

    fn server_tx(&self, msg: &[u8]) {
        if let Some(cb) = self.server_tx.borrow_mut().as_mut() {
            cb(msg.to_vec());
        } else {
            tracing::error!("Server tx not registered");
        }
    }

    fn on_server_tx(&self, cb: Box<dyn Fn(Vec<u8>)>) {
        if let Some(cb) = self.server_tx.borrow_mut().as_mut() {
            panic!("Server tx already registered");
        }
        *self.server_tx.borrow_mut() = Some(cb);
    }

    fn on_server_rx(&self, cb: Box<dyn Fn(Vec<u8>)>) {
        if let Some(cb) = self.server_rx.borrow_mut().as_mut() {
            panic!("Server rx already registered");
        }
        *self.server_rx.borrow_mut() = Some(cb);
    }

    fn client_rx(&self, msg: &[u8]) {
        if let Some(cb) = self.client_rx.borrow_mut().as_mut() {
            cb(msg.to_vec());
        } else {
            tracing::error!("Client rx not registered");
        }
    }

    fn client_tx(&self, msg: &[u8]) {
        if let Some(cb) = self.client_tx.borrow_mut().as_mut() {
            cb(msg.to_vec());
        } else {
            tracing::error!("Client tx not registered");
        }
    }

    fn on_client_tx(&self, cb: Box<dyn Fn(Vec<u8>)>) {
        if let Some(cb) = self.client_tx.borrow_mut().as_mut() {
            panic!("Client tx already registered");
        }
        *self.client_tx.borrow_mut() = Some(cb);
    }

    fn on_client_rx(&self, cb: Box<dyn Fn(Vec<u8>)>) {
        if let Some(cb) = self.client_rx.borrow_mut().as_mut() {
            panic!("Client rx already registered");
        }
        *self.client_rx.borrow_mut() = Some(cb);
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
            transport.on_client_rx(Box::new(move |bytes| {
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
        self.transport.client_tx(&msg);
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
        self.transport.client_tx(&bytes);
        let bytes = self.recv(id).await;
        let resp: Response = prost::Message::decode(bytes.as_slice()).unwrap();
        match resp.client_resp {
            Some(response::ClientResp::TableSizeResp(TableSizeResp { size, .. })) => size as usize,
            _ => panic!("Unexpected response"),
        }
    }

    fn register_as_server(&self, transport: Arc<dyn Transport>) {
        unimplemented!("Cannot register transport on remote client")
    }

    fn register_as_client(&self, transport: Arc<dyn Transport>) {
        unimplemented!("Cannot register transport on remote client")
    }
}

#[wasm_bindgen]
pub struct MemoryPerspectiveClient {
    tables: Arc<Mutex<HashMap<TableId, perspective_ffi::Table>>>,
}

#[wasm_bindgen]
impl MemoryPerspectiveClient {
    pub fn new() -> MemoryPerspectiveClient {
        MemoryPerspectiveClient {
            tables: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // #[wasm_bindgen(js_name = "registerTransport")]
    // async fn with_transport(transport: Arc<PerspectiveTransport>) -> MemoryPerspectiveClient {
    //     let tables = Arc::new(Mutex::new(HashMap::new()));
    //     {
    //         let tables = tables.clone();
    //         let write_transport = transport.clone();
    //         transport
    //             .registered
    //             .borrow_mut()
    //             .push(Box::new(move |bytes| {
    //                 let tables = tables.clone();
    //                 let write_transport = write_transport.clone();
    //                 spawn_local(async move {
    //                     let env = MultiplexEnvelope::decode(bytes.as_slice()).unwrap();

    //                     let resp: Request = prost::Message::decode(env.payload.as_slice()).unwrap();
    //                     match resp.client_req {
    //                         Some(ClientReq::MakeTableReq(MakeTableReq {})) => {
    //                             tables
    //                                 .lock()
    //                                 .await
    //                                 .insert(env.id, perspective_ffi::Table::new());
    //                         }
    //                         Some(ClientReq::TableSizeReq(TableSizeReq {})) => {
    //                             let size = tables.lock().await.get(&env.id).unwrap().size();
    //                             let resp = Response {
    //                                 client_resp: Some(response::ClientResp::TableSizeResp(
    //                                     TableSizeResp { size: size as u64 },
    //                                 )),
    //                             };
    //                             let bytes = prost::Message::encode_to_vec(&resp);
    //                             let envelope = MultiplexEnvelope {
    //                                 id: env.id,
    //                                 payload: bytes,
    //                             };
    //                             let bytes = prost::Message::encode_to_vec(&envelope);
    //                             write_transport.send(&bytes);
    //                         }
    //                         _ => todo!(),
    //                     }
    //                 });
    //             }));
    //     }
    //     Self { tables }
    // }
    #[wasm_bindgen]
    pub fn make() -> JsClient {
        JsClient(Arc::new(MemoryPerspectiveClient::new()))
    }

    // #[wasm_bindgen(js_name = "makeWithTransport")]
    // pub async fn make_with_transport(transport: &JsTransport) -> JsClient {
    //     JsClient(Arc::new(
    //         MemoryPerspectiveClient::with_transport(transport.0.clone()).await,
    //     ))
    // }
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

    #[wasm_bindgen(js_name = "registerServer")]
    pub fn register_server_js(&mut self, transport: &JsTransport) {
        self.0.register_as_server(transport.0.clone());
    }

    #[wasm_bindgen(js_name = "registerClient")]
    pub fn register_client_js(&mut self, transport: &JsTransport) {
        self.0.register_as_client(transport.0.clone());
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct JsTransport(Arc<PerspectiveTransport>);

#[wasm_bindgen]
impl JsTransport {
    #[wasm_bindgen(js_name = "onClientTx")]
    pub fn on_client_tx_js(&mut self, cb: js_sys::Function) {
        self.0.on_client_tx(Box::new(move |bytes| {
            let this = JsValue::UNDEFINED;
            let bytes = js_sys::Uint8Array::from(bytes.as_slice());
            let _ = cb.call1(&this, &bytes).unwrap();
        }));
    }

    #[wasm_bindgen(js_name = "onClientRx")]
    pub fn on_client_rx_js(&mut self, cb: js_sys::Function) {
        self.0.on_client_rx(Box::new(move |bytes| {
            let this = JsValue::UNDEFINED;
            let bytes = js_sys::Uint8Array::from(bytes.as_slice());
            let _ = cb.call1(&this, &bytes).unwrap();
        }));
    }

    #[wasm_bindgen(js_name = "onServerRx")]
    pub fn on_server_rx(&mut self, cb: js_sys::Function) {
        self.0.on_server_rx(Box::new(move |bytes| {
            let this = JsValue::UNDEFINED;
            let bytes = js_sys::Uint8Array::from(bytes.as_slice());
            let _ = cb.call1(&this, &bytes).unwrap();
        }));
    }

    #[wasm_bindgen(js_name = "onServerTx")]
    pub fn on_server_tx(&mut self, cb: js_sys::Function) {
        self.0.on_server_tx(Box::new(move |bytes| {
            let this = JsValue::UNDEFINED;
            let bytes = js_sys::Uint8Array::from(bytes.as_slice());
            let _ = cb.call1(&this, &bytes).unwrap();
        }));
    }

    #[wasm_bindgen(js_name = "clientRx")]
    pub fn client_rx(&self, bytes: &[u8]) {
        self.0.client_rx(bytes);
    }

    #[wasm_bindgen(js_name = "serverRx")]
    pub fn server_rx(&self, bytes: &[u8]) {
        self.0.server_rx(bytes);
    }

    #[wasm_bindgen(js_name = "clientTx")]
    pub fn client_tx(&self, bytes: &[u8]) {
        self.0.client_tx(bytes);
    }

    #[wasm_bindgen(js_name = "serverTx")]
    pub fn server_tx(&self, bytes: &[u8]) {
        self.0.server_tx(bytes);
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

    fn register_as_client(&self, transport: Arc<dyn Transport>) {
        unimplemented!("Cannot register transport on memory client")
    }

    fn register_as_server(&self, transport: Arc<dyn Transport>) {
        let tables = self.tables.clone();
        let write_transport = transport.clone();
        let id = Arc::new(RefCell::new(1));
        transport.on_server_rx(Box::new(move |bytes| {
            let tables = tables.clone();
            let id = id.clone();
            let write_transport = write_transport.clone();
            spawn_local(async move {
                let env = MultiplexEnvelope::decode(bytes.as_slice()).unwrap();

                let resp: Request = prost::Message::decode(env.payload.as_slice()).unwrap();
                match resp.client_req {
                    Some(ClientReq::MakeTableReq(MakeTableReq {})) => {
                        let table_id = *id.borrow();
                        *(*id).borrow_mut() += 1;
                        tracing::error!("Creating table with id: {}", table_id);
                        tables
                            .lock()
                            .await
                            .insert(table_id, perspective_ffi::Table::new());

                        let resp = Response {
                            client_resp: Some(response::ClientResp::MakeTableResp(MakeTableResp {
                                id: table_id,
                            })),
                        };
                        let bytes = prost::Message::encode_to_vec(&resp);
                        let envelope = MultiplexEnvelope {
                            id: env.id,
                            payload: bytes,
                        };
                        let bytes = prost::Message::encode_to_vec(&envelope);
                        write_transport.server_tx(&bytes);
                    }
                    Some(ClientReq::TableSizeReq(TableSizeReq {})) => {
                        let size = tables.lock().await.get(&env.id).unwrap().size();
                        let resp = Response {
                            client_resp: Some(response::ClientResp::TableSizeResp(TableSizeResp {
                                size: size as u64,
                            })),
                        };
                        let bytes = prost::Message::encode_to_vec(&resp);
                        let envelope = MultiplexEnvelope {
                            id: env.id,
                            payload: bytes,
                        };
                        let bytes = prost::Message::encode_to_vec(&envelope);
                        write_transport.server_tx(&bytes);
                    }
                    _ => todo!(),
                }
            });
        }));
    }
}
