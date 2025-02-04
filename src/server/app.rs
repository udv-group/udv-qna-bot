use askama::Template;
use axum::body::Body;
use axum::http::header;
use axum::response::Response;
use axum::{extract::FromRef, http::StatusCode, response::Html, routing::get, Router};
use prometheus::{Encoder, TextEncoder};
use routes::{category_router, questions_router, users_router};
use sqlx::SqlitePool;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use super::routes;

#[derive(FromRef, Clone)]
pub struct AppState {
    pool: SqlitePool,
    static_dir: PathBuf,
}

pub async fn run_server(pool: SqlitePool, static_dir: PathBuf) -> anyhow::Result<()> {
    let addr = "0.0.0.0:8080";
    let state = AppState {
        pool,
        static_dir: static_dir.clone(),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/metrics", get(metrics))
        .nest_service("/static", ServeDir::new(static_dir))
        .merge(category_router(state.clone()))
        .merge(questions_router(state.clone()))
        .merge(users_router(state.clone()))
        .fallback(|| async {
            tracing::info!("Fallback");
            StatusCode::NOT_FOUND
        })
        .layer(TraceLayer::new_for_http());
    let listener = TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Serving on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> Html<String> {
    let tmpl = IndexPage {};
    Html(tmpl.render().unwrap())
}

#[derive(Template)]
#[template(path = "index.html", escape = "none")]
struct IndexPage;

async fn metrics() -> Response {
    let encoder = TextEncoder::new();
    let metrics = prometheus::gather();
    let mut buf = vec![];
    encoder.encode(&metrics, &mut buf).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, encoder.format_type())
        .body(Body::from(buf))
        .unwrap()
}
