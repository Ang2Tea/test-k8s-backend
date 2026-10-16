use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::domain::{Product, ProductRepository};

#[derive(Clone)]
pub struct InMemoryStorage {
    storage: Arc<RwLock<HashMap<i32, Product>>>,
}

impl From<Vec<(i32, String, f64)>> for InMemoryStorage {
    fn from(value: Vec<(i32, String, f64)>) -> Self {
        let mut map = HashMap::with_capacity(value.len());

        for (id, title, price) in value {
            map.insert(id, Product { id, title, price });
        }

        Self {
            storage: Arc::new(RwLock::new(map)),
        }
    }
}

impl ProductRepository for InMemoryStorage {
    #[tracing::instrument(skip(self))]
    async fn get_list(&self, offset: u64, limit: u64) -> anyhow::Result<Vec<Product>> {
        let products = self
            .storage
            .read()
            .await
            .iter()
            .skip(offset as usize)
            .take(limit as usize)
            .map(|(_, p)| p.clone())
            .collect::<Vec<_>>();

        Ok(products)
    }

    #[tracing::instrument(skip(self))]
    async fn get_by_id(&self, id: i32) -> anyhow::Result<Product> {
        let product = self
            .storage
            .read()
            .await
            .get(&id)
            .ok_or(anyhow::anyhow!("not found by id {}", id))?
            .clone();

        Ok(product)
    }
}
