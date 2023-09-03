use askama::Template;
use axum::{
    extract::FromRef,
    http::StatusCode,
    response::Html,
    routing::get,
    Router,
};
use categories::category_router;
use sqlx::{Pool, Sqlite, SqlitePool};
use users::users_router;

mod categories;
mod questions;
mod users;

#[derive(FromRef, Clone)]
pub struct AppState {
    pool: SqlitePool,
}

#[tokio::main]
async fn main() {
    let path = dotenv::var("DB_PATH").expect("DB_PATH must be set");
    let pool = db::establish_connection(&path).await.unwrap();
    let state = AppState { pool };

    let app = Router::new()
        .route("/", get(index))
        .merge(category_router(state.clone()))
        .merge(users_router(state.clone()));
    let app = app.fallback(|| async { StatusCode::NOT_FOUND });

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
