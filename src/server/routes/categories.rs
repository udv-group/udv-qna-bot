use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;

use deserializers::deserialize_bool_from_checkbox;
use deserializers::Stri64;
use sqlx::SqlitePool;

use crate::{
    db::{
        queries::categories::{self, get_all_categories, get_category},
        Category, Reorder,
    },
    server::{app::AppState, deserializers},
};

use super::ApiResponse;

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

async fn get_categories(State(pool): State<SqlitePool>) -> ApiResponse<CatgeoriesPage> {
    let categories = get_all_categories(&pool).await?;
    Ok(CatgeoriesPage {
        categories: categories
            .into_iter()
            .map(|c| CategoryRow { category: c })
            .collect(),
    })
}

async fn category(State(pool): State<SqlitePool>, Path(id): Path<i64>) -> ApiResponse<CategoryRow> {
    let c = get_category(&pool, id).await?;
    Ok(CategoryRow { category: c })
}

async fn edit_category(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> ApiResponse<CategoryRowEdit> {
    let c = get_category(&pool, id).await?;
    Ok(CategoryRowEdit { category: c })
}

async fn create_category(
    State(pool): State<SqlitePool>,
    Json(new_category): Json<NewCategory>,
) -> ApiResponse<CategoryRow> {
    let id = categories::create_category(
        &pool,
        new_category.name.as_str(),
        new_category.hidden.unwrap_or(false),
        1,
    )
    .await?;

    Ok(CategoryRow {
        category: categories::get_category(&pool, id).await?,
    })
}

async fn update_category(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(category): Json<CategoryUpdate>,
) -> ApiResponse<CategoryRow> {
    categories::update_category(&pool, id, category.name, category.hidden.unwrap_or(false)).await?;
    Ok(CategoryRow {
        category: categories::get_category(&pool, id).await?,
    })
}

async fn delete_category(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> ApiResponse<StatusCode> {
    categories::delete_category(&pool, id).await?;
    Ok(StatusCode::OK)
}

async fn render_reordering_page(
    State(pool): State<SqlitePool>,
) -> ApiResponse<CatgeoriesReorderingPage> {
    let categories = categories::get_all_categories(&pool).await?;
    Ok(CatgeoriesReorderingPage { categories })
}

async fn reorder(
    State(pool): State<SqlitePool>,
    Json(body): Json<OrderingBody>,
) -> ApiResponse<CatgeoriesPage> {
    let ordering: Vec<Reorder> = body
        .row_id
        .into_iter()
        .enumerate()
        .map(|(n, v)| Reorder {
            id: v.0,
            ordering: n as i64,
        })
        .collect();

    categories::reorder_categories(&pool, ordering).await?;

    let categories = categories::get_all_categories(&pool).await?;
    Ok(CatgeoriesPage {
        categories: categories
            .into_iter()
            .map(|c| CategoryRow { category: c })
            .collect(),
    })
}

pub fn category_router(state: AppState) -> Router {
    Router::new()
        .route("/categories", get(get_categories))
        .route("/categories/new", post(create_category))
        .route(
            "/categories/:id",
            delete(delete_category).put(update_category).get(category),
        )
        .route("/categories/:id/edit", get(edit_category))
        .route(
            "/categories/order",
            get(render_reordering_page).post(reorder),
        )
        .with_state(state)
}
