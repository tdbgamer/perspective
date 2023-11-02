use std::sync::Arc;

use async_trait::async_trait;

pub type TableId = u32;
pub type ViewId = u32;

#[async_trait(?Send)]
pub trait Transport {
    async fn send(&self, msg: &[u8]);
    async fn on_message(&self, cb: Box<dyn Fn(Vec<u8>)>);
}

#[async_trait(?Send)]
pub trait PerspectiveClient {
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
