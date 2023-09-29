use crate::deserializers::deserialize_bool_from_checkbox;
use crate::AppState;
use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use db::Category;
use serde::Deserialize;
use sqlx::SqlitePool;

use super::Stri64;

#[derive(Deserialize)]
struct NewCategory {
    name: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_bool_from_checkbox")]
    hidden: Option<bool>,
}

#[derive(Deserialize)]
struct CategoryUpdate {
    name: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_bool_from_checkbox")]
    hidden: Option<bool>,
}

// a hack to deserialize array of strings to Vec<i64>, since this is how it's get encoded and
// I don't want to write JS to do it on frontend
#[derive(Deserialize)]
struct OrderingBody {
    row_id: Vec<Stri64>,
}

#[derive(Template)]
#[template(path = "categories/category_row.html", escape = "none")]
struct CategoryRow {
    category: Category,
}

#[derive(Template)]
#[template(path = "categories/category_row_edit.html", escape = "none")]
struct CategoryRowEdit {
    category: Category,
}

#[derive(Template)]
#[template(path = "categories/categories.html", escape = "none")]
struct CatgeoriesPage {
    categories: Vec<CategoryRow>,
}

#[derive(Template)]
#[template(path = "categories/categories_reordering.html", escape = "none")]
struct CatgeoriesReorderingPage {
    categories: Vec<Category>,
}

async fn get_categories(State(pool): State<SqlitePool>) -> impl IntoResponse {
    let categories = db::categories::get_all_categories(&pool).await.unwrap();
    CatgeoriesPage {
        categories: categories
            .into_iter()
            .map(|c| CategoryRow { category: c })
            .collect(),
    }
}

async fn get_category(State(pool): State<SqlitePool>, Path(id): Path<i64>) -> impl IntoResponse {
    let c = db::categories::get_category(&pool, id).await.unwrap();
    CategoryRow { category: c }
}

async fn edit_category(State(pool): State<SqlitePool>, Path(id): Path<i64>) -> impl IntoResponse {
    let c = db::categories::get_category(&pool, id).await.unwrap();
    CategoryRowEdit { category: c }
}

async fn create_category(
    State(pool): State<SqlitePool>,
    Json(new_category): Json<NewCategory>,
) -> impl IntoResponse {
    let id = db::categories::create_category(
        &pool,
        new_category.name.as_str(),
        new_category.hidden.unwrap_or(false),
        1,
    )
    .await
    .unwrap();

    CategoryRow {
        category: db::categories::get_category(&pool, id).await.unwrap(),
    }
}

async fn update_category(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(category): Json<CategoryUpdate>,
) -> impl IntoResponse {
    db::categories::update_category(&pool, id, category.name, category.hidden.unwrap_or(false))
        .await
        .unwrap();
    CategoryRow {
        category: db::categories::get_category(&pool, id).await.unwrap(),
    }
}

async fn delete_category(State(pool): State<SqlitePool>, Path(id): Path<i64>) -> impl IntoResponse {
    db::categories::delete_category(&pool, id).await.unwrap();
    StatusCode::OK
}

async fn render_reordering_page(State(pool): State<SqlitePool>) -> impl IntoResponse {
    let categories = db::categories::get_all_categories(&pool).await.unwrap();
    CatgeoriesReorderingPage { categories }
}

async fn reorder(
    State(pool): State<SqlitePool>,
    Json(body): Json<OrderingBody>,
) -> impl IntoResponse {
    let ordering: Vec<db::Reorder> = body
        .row_id
        .into_iter()
        .enumerate()
        .map(|(n, v)| db::Reorder {
            id: v.0,
            ordering: n as i64,
        })
        .collect();

    db::categories::reorder_categories(&pool, ordering)
        .await
        .unwrap();

    let categories = db::categories::get_all_categories(&pool).await.unwrap();
    CatgeoriesPage {
        categories: categories
            .into_iter()
            .map(|c| CategoryRow { category: c })
            .collect(),
    }
}

pub fn category_router(state: AppState) -> Router {
    Router::new()
        .route("/categories", get(get_categories))
        .route("/categories/new", post(create_category))
        .route(
            "/categories/:id",
            delete(delete_category)
                .put(update_category)
                .get(get_category),
        )
        .route("/categories/:id/edit", get(edit_category))
        .route(
            "/categories/order",
            get(render_reordering_page).post(reorder),
        )
        .with_state(state)
}
