use serde::{Deserialize, Serialize};

use crate::domain::Product;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductDto {
    pub id: i32,
    pub name: String,
    pub price: f64,
}

impl From<Product> for ProductDto {
    fn from(value: Product) -> Self {
        Self {
            id: value.id,
            name: value.title,
            price: value.price,
        }
    }
}

fn default_limit() -> u64 {
    10
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationQueryDto {
    #[serde(default)]
    pub offset: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
}

#[derive(Serialize, Deserialize)]
pub struct PaginationResultDto<T> {
    pub data: Vec<T>,
    pub total: u64,
}

impl<T> PaginationResultDto<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self {
            total: data.len() as u64,
            data: data,
        }
    }
}
