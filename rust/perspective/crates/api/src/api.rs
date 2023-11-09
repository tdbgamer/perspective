use std::{collections::HashMap, hash::Hash, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// pub mod view_config;

pub type TableId = u32;
pub type ViewId = u32;

pub trait Transport {
    fn tx(&self, msg: &[u8]);
    fn rx(&self, msg: &[u8]);
    fn on_tx(&self, cb: Box<dyn Fn(Vec<u8>)>);
    fn on_rx(&self, cb: Box<dyn Fn(Vec<u8>)>);
}

// Each of these methods should be aware of when `process` needs to occur.
#[async_trait(?Send)]
pub trait PerspectiveClient {
    async fn table_size(&self, id: TableId) -> usize;
    async fn table_schema(&self, id: TableId) -> Schema;
    async fn table_make_port(&self, id: TableId) -> usize;
    async fn make_table(self: Arc<Self>) -> Table;
    // Probably shouldn't be in the public API given ^
    async fn process(&self);
}

// #[async_trait(?Send)]
// pub trait PerspectiveServer {
//     async fn table_size(&self, id: TableId) -> usize;
//     async fn make_table(&self) -> Table;
// }

pub struct Schema {
    pub fields: HashMap<String, u8>,
}

impl Into<HashMap<String, u32>> for Schema {
    fn into(self) -> HashMap<String, u32> {
        self.fields
            .into_iter()
            .map(|(k, v)| (k, v as u32))
            .collect()
    }
}

impl Into<HashMap<String, u8>> for Schema {
    fn into(self) -> HashMap<String, u8> {
        self.fields
    }
}

impl From<HashMap<String, u8>> for Schema {
    fn from(fields: HashMap<String, u8>) -> Self {
        Schema { fields }
    }
}

impl From<HashMap<String, u32>> for Schema {
    fn from(fields: HashMap<String, u32>) -> Self {
        let fields = fields.into_iter().map(|(k, v)| (k, v as u8)).collect();
        Schema { fields }
    }
}

pub struct Table {
    id: TableId,
    client: Arc<dyn PerspectiveClient>,
}

impl Table {
    pub fn new(id: TableId, client: Arc<dyn PerspectiveClient>) -> Self {
        Table { id, client }
    }
    pub async fn size(&self) -> usize {
        self.client.table_size(self.id).await
    }
    pub async fn schema(&self) -> Schema {
        self.client.table_schema(self.id).await
    }
    pub async fn columns(&self) -> Vec<String> {
        self.schema().await.fields.keys().cloned().collect()
    }
    pub async fn make_port(&self) -> usize {
        self.client.table_make_port(self.id).await
    }
}
