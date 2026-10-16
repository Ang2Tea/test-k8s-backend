mod domain;
mod infrastructure;
mod logger;
mod models;
mod shared;

use std::env;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use tower_http::trace::TraceLayer;

use crate::{
    domain::ProductRepository,
    infrastructure::InMemoryStorage,
    models::{PaginationQueryDto, PaginationResultDto, ProductDto},
    shared::AppError,
};

#[derive(Clone)]
struct DiContainer<PR> {
    product_repository: PR,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::from_path(".env");
    let port = env::var("APP_PORT")?.parse::<u16>()?;

    let _guard = logger::init()?;
    tracing::info!("Старт веб-сервера на Axum...");

    let storage = InMemoryStorage::from(vec![
        (1, "Продукт #1".to_owned(), 10.0),
        (2, "Продукт #2".to_owned(), 11.0),
        (3, "Продукт #3".to_owned(), 12.0),
        (4, "Продукт #4".to_owned(), 13.0),
    ]);

    let di = DiContainer {
        product_repository: storage,
    };

    let app = Router::new()
        .route("/products", get(get_list))
        .route("/products/{id}", get(get_by_id))
        .layer(TraceLayer::new_for_http())
        .with_state(di.clone());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;

    let _ = axum::serve(listener, app).await?;
    Ok(())
}

#[tracing::instrument(skip(di))]
async fn get_list<PR>(
    Query(query): Query<PaginationQueryDto>,
    State(di): State<DiContainer<PR>>,
) -> Result<Json<PaginationResultDto<ProductDto>>, AppError>
where
    PR: ProductRepository + Send + Sync + Clone + 'static,
{
    let products = di
        .product_repository
        .get_list(query.offset, query.limit)
        .await?
        .into_iter()
        .map(ProductDto::from)
        .collect();

    let result = PaginationResultDto::new(products);

    Ok(Json(result))
}

#[tracing::instrument(skip(di))]
async fn get_by_id<PR>(
    Path(id): Path<i32>,
    State(di): State<DiContainer<PR>>,
) -> Result<Json<ProductDto>, AppError>
where
    PR: ProductRepository + Send + Sync + Clone + 'static,
{
    let product = di.product_repository.get_by_id(id).await?.into();

    Ok(Json(product))
}
