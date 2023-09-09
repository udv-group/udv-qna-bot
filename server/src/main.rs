use askama::Template;
use axum::{extract::FromRef, http::StatusCode, response::Html, routing::get, Router};
use routes::{category_router, questions_router, users_router};
use sqlx::SqlitePool;

use std::fs::create_dir;
use std::path::PathBuf;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

mod deserializers;
mod routes;

#[derive(FromRef, Clone)]
pub struct AppState {
    pool: SqlitePool,
    static_dir: PathBuf,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let path = dotenv::var("DB_PATH").expect("DB_PATH must be set");
    let pool = db::establish_connection(&path).await.unwrap();
    let static_dir = PathBuf::from("static");
    if !static_dir.exists() {
        create_dir(&static_dir).unwrap();
    }
    let _static_service = ServeDir::new(&static_dir);

    let state = AppState { pool, static_dir };

    let app = Router::new()
        .route("/", get(index))
        .nest_service("/static", ServeDir::new("static"))
        .merge(category_router(state.clone()))
        .merge(questions_router(state.clone()))
        .merge(users_router(state.clone()))
        .fallback(|| async {
            tracing::info!("Fallback");
            StatusCode::NOT_FOUND
        })
        .layer(TraceLayer::new_for_http());

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Html<String> {
    let tmpl = IndexPage {};
    Html(tmpl.render().unwrap())
}

#[derive(Template)]
#[template(path = "index.html", escape = "none")]
struct IndexPage;
