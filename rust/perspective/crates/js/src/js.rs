use js_sys::Array;
use perspective_api::*;
use perspective_ffi::DType;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    env,
    sync::Arc,
};

use async_trait::async_trait;

use prost::Message;
use protos::{request::ClientReq, *};
use tokio::sync::Mutex;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;

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
    rx: RefCell<Option<Box<dyn Fn(Vec<u8>)>>>,
    tx: RefCell<Option<Box<dyn Fn(Vec<u8>)>>>,
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
    fn rx(&self, msg: &[u8]) {
        if let Some(cb) = self.rx.borrow_mut().as_mut() {
            cb(msg.to_vec());
        } else {
            tracing::error!("Server rx not registered");
        }
    }

    fn tx(&self, msg: &[u8]) {
        if let Some(cb) = self.tx.borrow_mut().as_mut() {
            cb(msg.to_vec());
        } else {
            tracing::error!("Server tx not registered");
        }
    }

    fn on_tx(&self, cb: Box<dyn Fn(Vec<u8>)>) {
        if let Some(_cb) = self.tx.borrow_mut().as_mut() {
            panic!("Server tx already registered");
        }
        *self.tx.borrow_mut() = Some(cb);
    }

    fn on_rx(&self, cb: Box<dyn Fn(Vec<u8>)>) {
        if let Some(_cb) = self.rx.borrow_mut().as_mut() {
            panic!("Server rx already registered");
        }
        *self.rx.borrow_mut() = Some(cb);
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
            transport.on_rx(Box::new(move |bytes| {
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
        let bytes = pack_envelope(0, req);
        self.transport.tx(&bytes);
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
        let bytes = pack_envelope(id, req);
        self.transport.tx(&bytes);
        let bytes = self.recv(id).await;
        let resp: Response = prost::Message::decode(bytes.as_slice()).unwrap();
        match resp.client_resp {
            Some(response::ClientResp::TableSizeResp(TableSizeResp { size, .. })) => size as usize,
            _ => panic!("Unexpected response"),
        }
    }

    async fn table_make_port(&self, id: TableId) -> usize {
        let req = Request {
            client_req: Some(request::ClientReq::TableMakePortReq(TableMakePortReq {})),
        };
        let bytes = pack_envelope(id, req);
        self.transport.tx(&bytes);
        let bytes = self.recv(id).await;
        let resp: Response = prost::Message::decode(bytes.as_slice()).unwrap();
        match resp.client_resp {
            Some(response::ClientResp::TableMakePortResp(TableMakePortResp { port_id })) => {
                port_id as usize
            }
            _ => panic!("Unexpected response"),
        }
    }

    async fn table_schema(&self, id: TableId) -> perspective_api::Schema {
        let req = Request {
            client_req: Some(request::ClientReq::TableSchemaReq(TableSchemaReq {})),
        };
        let bytes = pack_envelope(id, req);
        self.transport.tx(&bytes);
        let bytes = self.recv(id).await;
        let resp: Response = prost::Message::decode(bytes.as_slice()).unwrap();
        match resp.client_resp {
            Some(response::ClientResp::TableSchemaResp(TableSchemaResp { schema })) => {
                perspective_api::Schema::from(schema)
            }
            _ => panic!("Unexpected response"),
        }
    }

    async fn process(&self) {}
}

// TODO: Move this to the ffi crate?
#[wasm_bindgen]
#[derive(Clone)]
pub struct MemoryPerspectiveClient {
    tables: Arc<Mutex<HashMap<TableId, perspective_ffi::Table>>>,
}

pub fn pack_envelope<A: prost::Message>(id: TableId, msg: A) -> Vec<u8> {
    let envelope = MultiplexEnvelope {
        id,
        payload: msg.encode_to_vec(),
    };
    prost::Message::encode_to_vec(&envelope)
}

pub fn unpack_envelope<A: prost::Message + Default>(bytes: &[u8]) -> (TableId, A) {
    let envelope = MultiplexEnvelope::decode(bytes).unwrap();
    let msg = A::decode(envelope.payload.as_slice()).unwrap();
    (envelope.id, msg)
}

impl MemoryPerspectiveClient {
    /// Create a memory client that acts as a "server" for a remote client.
    pub fn with_transport(transport: Arc<PerspectiveTransport>) -> MemoryPerspectiveClient {
        let tables = Arc::new(Mutex::new(HashMap::new()));
        let client = MemoryPerspectiveClient {
            tables: tables.clone(),
        };
        let write_transport = transport.clone();
        let id = Arc::new(RefCell::new(1));
        {
            let client = client.clone();
            transport.on_rx(Box::new(move |bytes| {
                let tables = tables.clone();
                let client = client.clone();
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
                                client_resp: Some(response::ClientResp::MakeTableResp(
                                    MakeTableResp { id: table_id },
                                )),
                            };
                            let bytes = pack_envelope(env.id, resp);
                            write_transport.tx(&bytes);
                        }
                        Some(ClientReq::TableSizeReq(TableSizeReq {})) => {
                            let size = tables.lock().await.get(&env.id).unwrap().size();
                            let resp = Response {
                                client_resp: Some(response::ClientResp::TableSizeResp(
                                    TableSizeResp { size: size as u64 },
                                )),
                            };
                            let bytes = pack_envelope(env.id, resp);
                            write_transport.tx(&bytes);
                        }
                        Some(ClientReq::TableSchemaReq(TableSchemaReq {})) => {
                            let schema = client.table_schema(env.id).await;
                            let resp = Response {
                                client_resp: Some(response::ClientResp::TableSchemaResp(
                                    TableSchemaResp {
                                        schema: schema.into(),
                                    },
                                )),
                            };
                            let bytes = pack_envelope(env.id, resp);
                            write_transport.tx(&bytes);
                        }
                        Some(ClientReq::TableMakePortReq(TableMakePortReq {})) => {
                            let port_id = client.table_make_port(env.id).await;
                            let resp = Response {
                                client_resp: Some(response::ClientResp::TableMakePortResp(
                                    TableMakePortResp {
                                        port_id: port_id as u64,
                                    },
                                )),
                            };
                            let bytes = pack_envelope(env.id, resp);
                            write_transport.tx(&bytes);
                        }
                        _ => todo!(),
                    }
                });
            }));
        }
        client
    }
}

#[wasm_bindgen]
impl MemoryPerspectiveClient {
    pub fn new() -> MemoryPerspectiveClient {
        MemoryPerspectiveClient {
            tables: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[wasm_bindgen]
    pub fn make() -> JsClient {
        JsClient(Arc::new(MemoryPerspectiveClient::new()))
    }

    #[wasm_bindgen(js_name = "makeWithTransport")]
    pub fn make_with_transport(transport: &JsTransport) -> JsClient {
        JsClient(Arc::new(MemoryPerspectiveClient::with_transport(
            transport.0.clone(),
        )))
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

    #[wasm_bindgen(js_name = "process")]
    pub async fn process_js(&self) {
        self.0.process().await;
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct JsTransport(Arc<PerspectiveTransport>);

#[wasm_bindgen]
impl JsTransport {
    #[wasm_bindgen(js_name = "onTx")]
    pub fn on_tx_js(&mut self, cb: js_sys::Function) {
        self.0.on_tx(Box::new(move |bytes| {
            let this = JsValue::UNDEFINED;
            let bytes = js_sys::Uint8Array::from(bytes.as_slice());
            let _ = cb.call1(&this, &bytes).unwrap();
        }));
    }

    #[wasm_bindgen(js_name = "onRx")]
    pub fn on_rx_js(&mut self, cb: js_sys::Function) {
        self.0.on_rx(Box::new(move |bytes| {
            let this = JsValue::UNDEFINED;
            let bytes = js_sys::Uint8Array::from(bytes.as_slice());
            let _ = cb.call1(&this, &bytes).unwrap();
        }));
    }

    #[wasm_bindgen(js_name = "rx")]
    pub fn rx_js(&self, bytes: &[u8]) {
        self.0.rx(bytes);
    }

    #[wasm_bindgen(js_name = "tx")]
    pub fn tx_js(&self, bytes: &[u8]) {
        self.0.tx(bytes);
    }
}

#[wasm_bindgen]
pub struct JsTable(perspective_api::Table);

#[derive(Serialize, Deserialize)]
pub struct ValidatedExpr {
    #[serde(with = "serde_wasm_bindgen::preserve")]
    expression_schema: js_sys::Object,
    #[serde(with = "serde_wasm_bindgen::preserve")]
    expression_alias: js_sys::Object,
    #[serde(with = "serde_wasm_bindgen::preserve")]
    errors: js_sys::Object,
}

#[derive(Serialize, Deserialize)]
pub struct ViewDimensions {
    num_table_rows: usize,
    num_table_columns: usize,
    num_view_rows: usize,
    num_view_columns: usize,
}

#[wasm_bindgen]
impl JsTable {
    #[wasm_bindgen(js_name = "size")]
    pub async fn size(&self) -> usize {
        self.0.size().await
    }

    #[wasm_bindgen(js_name = "schema")]
    pub async fn schema(&self) -> JsValue {
        let schema = self.0.schema().await;
        let js_object = js_sys::Object::new();

        for (col, dtype_u8) in schema.fields.into_iter() {
            let dtype: DType = unsafe { std::mem::transmute(dtype_u8) };
            // TODO: Move this to api. This isn't JS only, this is the "perspective" type for all FFIs.
            let tpe = match dtype {
                DType::DTYPE_NONE => "null",
                DType::DTYPE_INT64 => "number",
                DType::DTYPE_INT32 => "number", // * only these are used in js/python
                DType::DTYPE_INT16 => "number",
                DType::DTYPE_INT8 => "number",
                DType::DTYPE_UINT64 => "number",
                DType::DTYPE_UINT32 => "number",
                DType::DTYPE_UINT16 => "number",
                DType::DTYPE_UINT8 => "number",
                DType::DTYPE_FLOAT64 => "number", // * ^ same
                DType::DTYPE_FLOAT32 => "number",
                DType::DTYPE_BOOL => "boolean",
                DType::DTYPE_TIME => "number",
                DType::DTYPE_DATE => "date",
                DType::DTYPE_ENUM => todo!(), // Make these error, none of the todo's are used.
                DType::DTYPE_OID => todo!(),
                DType::DTYPE_OBJECT => todo!(),
                DType::DTYPE_F64PAIR => todo!(),
                DType::DTYPE_USER_FIXED => todo!(),
                DType::DTYPE_STR => "string",
                DType::DTYPE_USER_VLEN => todo!(),
                DType::DTYPE_LAST_VLEN => todo!(),
                DType::DTYPE_LAST => todo!(),
                _ => "unknown",
            };
            js_sys::Reflect::set(&js_object, &col.into(), &tpe.into()).unwrap();
        }

        js_object.unchecked_into()
    }

    #[wasm_bindgen(js_name = "makePort")]
    pub async fn make_port(&self) -> usize {
        self.0.make_port().await
    }

    #[wasm_bindgen(js_name = "columns")]
    pub async fn columns(&self) -> js_sys::Array {
        serde_wasm_bindgen::to_value(&self.0.columns().await)
            .unwrap()
            .into()
    }
}

#[async_trait(?Send)]
impl PerspectiveClient for MemoryPerspectiveClient {
    async fn table_size(&self, id: TableId) -> usize {
        self.tables.lock().await.get(&id).unwrap().size()
    }
    async fn table_schema(&self, id: TableId) -> perspective_api::Schema {
        let schema = self.tables.lock().await.get(&id).unwrap().schema();
        let mut map = HashMap::new();
        for (col, dtype) in schema.columns().into_iter().zip(schema.types().into_iter()) {
            map.insert(col, dtype.repr);
        }
        perspective_api::Schema { fields: map }
    }
    async fn table_make_port(&self, id: TableId) -> usize {
        self.tables.lock().await.get(&id).unwrap().make_port()
    }
    async fn make_table(self: Arc<Self>) -> perspective_api::Table {
        let table = perspective_ffi::Table::new();
        self.tables.lock().await.insert(0, table);
        perspective_api::Table::new(0, self)
    }
    async fn process(&self) {
        self.tables.lock().await.iter().for_each(|(_, t)| {
            t.process();
        });
    }
}
