use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use db::Category;
use serde::Deserialize;
use sqlx::{Pool, Sqlite, SqlitePool};

#[derive(Deserialize)]
struct NewCategory {
    name: String,
    hidden: bool,
}

async fn get_categories(State(pool): State<SqlitePool>) -> Json<Vec<Category>> {
    Json(db::categories::get_all_categories(&pool).await.unwrap())
}

async fn create_category(
    State(pool): State<SqlitePool>,
    Json(new_category): Json<NewCategory>,
) -> impl IntoResponse {
    let id = db::categories::create_category(&pool, new_category.name.as_str(), new_category.hidden)
        .await
        .unwrap();
    Json(db::categories::get_category(&pool, id).await.unwrap())
}

async fn update_category(
    State(pool): State<SqlitePool>,
    Json(category): Json<Category>,
) -> impl IntoResponse {
    let id = category.id;
    db::categories::update_category(&pool, category)
        .await
        .unwrap();
    Json(db::categories::get_category(&pool, id).await.unwrap())
}

async fn delete_category(State(pool): State<SqlitePool>, Path(id): Path<i64>) -> impl IntoResponse {
    db::categories::delete_category(&pool, id).await.unwrap();
    StatusCode::OK
}

pub fn category_router(pool: Pool<Sqlite>) -> Router {
    Router::new()
        .route("/categories", get(get_categories).put(update_category))
        .route("/categories/new", post(create_category))
        .route("/categories/:id", delete(delete_category))
        .with_state(pool)
}
