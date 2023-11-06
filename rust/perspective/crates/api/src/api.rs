use std::sync::Arc;

use async_trait::async_trait;

pub type TableId = u32;
pub type ViewId = u32;

pub trait Transport {
    fn server_tx(&self, msg: &[u8]);
    fn server_rx(&self, msg: &[u8]);
    fn on_server_tx(&self, cb: Box<dyn Fn(Vec<u8>)>);
    fn on_server_rx(&self, cb: Box<dyn Fn(Vec<u8>)>);
    fn client_tx(&self, msg: &[u8]);
    fn client_rx(&self, msg: &[u8]);
    fn on_client_tx(&self, cb: Box<dyn Fn(Vec<u8>)>);
    fn on_client_rx(&self, cb: Box<dyn Fn(Vec<u8>)>);
}

#[async_trait(?Send)]
pub trait PerspectiveClient {
    fn register_as_server(&self, transport: Arc<dyn Transport>);
    fn register_as_client(&self, transport: Arc<dyn Transport>);
    async fn table_size(&self, id: TableId) -> usize;
    async fn make_table(self: Arc<Self>) -> Table;
}

#[async_trait(?Send)]
pub trait PerspectiveServer {
    async fn table_size(&self, id: TableId) -> usize;
    async fn make_table(&self) -> Table;
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
}
