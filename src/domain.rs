#[derive(Debug, Clone)]
pub struct Product {
    pub id: i32,
    pub title: String,
    pub price: f64,
}

pub trait ProductRepository {
    fn get_list(
        &self,
        offset: u64,
        limit: u64,
    ) -> impl Future<Output = anyhow::Result<Vec<Product>>> + Send;
    fn get_by_id(&self, id: i32) -> impl Future<Output = anyhow::Result<Product>> + Send;
}
