use perspective_api::*;
use perspective_ffi::{Pool, Table};
use wasm_bindgen::prelude::*;

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

pub mod protos {
    include!(concat!(env!("OUT_DIR"), "/perspective.proto.rs"));
}
use prost::Message;
use protos::*;
use tokio::sync::Mutex;
use wasm_bindgen::{prelude::*, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::WebSocket;

// WASI reactor initialization function.
// This is because we're not running a `main` entrypoint that
// results in an exit code, we're initializing a library such that
// we can consume its functions from JavaScript.
#[no_mangle]
extern "C" fn _initialize() {
    println!("Hello, world!");
}

#[wasm_bindgen]
pub struct WasmWebSocketTransport {
    ws: Arc<WebSocket>,
    _cb: Closure<dyn FnMut(web_sys::MessageEvent)>,
    recv_buffer: Arc<Mutex<HashMap<Id, Vec<u8>>>>,
}

#[wasm_bindgen]
impl WasmWebSocketTransport {
    #[wasm_bindgen(constructor)]
    pub async fn new(url: &str) -> Result<WasmWebSocketTransport, JsValue> {
        let ws = WebSocket::new(url)?;

        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let recv_buffer = Arc::new(Mutex::new(HashMap::<Id, Vec<u8>>::new()));
        let recv_buffer_clone = recv_buffer.clone();

        let onmessage_callback = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            let recv_buffer_clone = recv_buffer_clone.clone();
            spawn_local(async move {
                if let Ok(ab) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                    let bytes = js_sys::Uint8Array::new(&ab).to_vec();
                    let env = MultiplexEnvelope::decode(bytes.as_slice()).unwrap();
                    let mut recv_buffer = recv_buffer_clone.lock().await;

                    if let Some(buf) = recv_buffer.get_mut(&env.id) {
                        buf.extend(&env.payload);
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
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));

        onopen_rx.await.unwrap();

        ws.set_onopen(None);
        drop(onopen_callback);

        Ok(WasmWebSocketTransport {
            ws: Arc::new(ws),
            // Only GC the callback when the Transport is dropped
            _cb: onmessage_callback,
            recv_buffer,
        })
    }
}

#[async_trait(?Send)]
impl Transport for WasmWebSocketTransport {
    async fn send(&self, id: Id, msg: Vec<u8>) {
        let env = MultiplexEnvelope { id, payload: msg };
        let mut bytes = Vec::new();
        env.encode(&mut bytes).unwrap();
        self.ws.send_with_u8_array(&bytes).unwrap();
    }
    async fn recv(&self, id: Id) -> Vec<u8> {
        let mut recv_buffer = self.recv_buffer.lock().await;
        if let Some(buf) = recv_buffer.get_mut(&id) {
            buf.drain(..).collect()
        } else {
            recv_buffer.insert(id, Vec::new());
            Vec::new()
        }
    }
}

pub struct RemotePerspectiveClient {
    transport: Box<dyn Transport>,
}
impl RemotePerspectiveClient {
    fn new(transport: Box<dyn Transport>) -> Self {
        RemotePerspectiveClient { transport }
    }
}

#[async_trait(?Send)]
impl PerspectiveClient for RemotePerspectiveClient {
    async fn make_table(self: Arc<Self>) -> perspective_api::Table {
        unimplemented!("")
        //     self.transport
        //         .send(id, ClientCommand::MakeTable(Schema::new()).to_bytes())
        //         .await;
        //     let id = self.transport.recv(id).await;
        //     Table {
        //         id,
        //         client: Arc::new(self.clone()),
        //     }
        // }
    }
    async fn table_size(&self, id: Id) -> u32 {
        let req = TableReq {
            table_req: Some(table_req::TableReq::Size(TableSizeReq { id })),
        };
        let mut bytes = Vec::new();
        req.encode(&mut bytes).unwrap();
        self.transport.send(id, bytes).await;
        let bytes = self.transport.recv(id).await;
        let b = prost::bytes::Bytes::from(bytes);
        let resp = TableResp::decode(b).unwrap();
        match resp {
            TableResp {
                table_resp: Some(table_resp::TableResp::Size(TableSizeResp { size, .. })),
            } => size,
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
pub struct JsTable(perspective_api::Table);

#[wasm_bindgen]
impl JsTable {
    #[wasm_bindgen(js_name = "size")]
    pub async fn size(&self) -> u32 {
        self.0.size().await
    }
}

#[async_trait(?Send)]
impl PerspectiveClient for MemoryPerspectiveClient {
    async fn table_size(&self, id: Id) -> u32 {
        self.tables.lock().await.get(&id).unwrap().size()
    }
    async fn make_table(self: Arc<Self>) -> perspective_api::Table {
        let table = perspective_ffi::Table::new();
        self.tables.lock().await.insert(0, table);
        perspective_api::Table::new(0, self)
    }
}
