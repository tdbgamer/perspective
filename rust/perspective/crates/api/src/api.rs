use std::sync::Arc;

use async_trait::async_trait;

pub type Id = u32;

#[async_trait(?Send)]
pub trait Transport {
    async fn send(&self, id: Id, msg: Vec<u8>);
    async fn recv(&self, id: Id) -> Vec<u8>;
}

#[async_trait(?Send)]
pub trait PerspectiveClient {
    async fn table_size(&self, id: Id) -> usize;
    async fn make_table(self: Arc<Self>) -> Table;
}

pub struct Table {
    id: Id,
    client: Arc<dyn PerspectiveClient>,
}

impl Table {
    pub fn new(id: Id, client: Arc<dyn PerspectiveClient>) -> Self {
        Table { id, client }
    }
    pub async fn size(&self) -> usize {
        self.client.table_size(self.id).await
    }
}
