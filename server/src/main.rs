use axum::{http::StatusCode, Router};
use categories::category_router;
use users::users_router;

mod categories;
mod users;

#[tokio::main]
async fn main() {
    let path = dotenv::var("DB_PATH").expect("DB_PATH must be set");
    let pool = db::establish_connection(&path).await.unwrap();

    let app = Router::new()
        .merge(category_router(pool.clone()))
        .merge(users_router(pool.clone()));
    let app = app.fallback(|| async { StatusCode::NOT_FOUND });

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
