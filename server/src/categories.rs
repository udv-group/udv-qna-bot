use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{delete, get, post},
    Form, Json, Router,
};
use db::Category;
use serde::{Deserialize, Deserializer};
use sqlx::{Pool, Sqlite, SqlitePool};

use crate::AppState;
use askama::Template;

fn deserialize_bool_from_checkbox<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    if value.is_none() {
        return Ok(None);
    } else {
        match value.unwrap().as_str() {
            "on" => Ok(Some(true)),
            variant => Err(serde::de::Error::unknown_variant(variant, &["on"])),
        }
    }
}

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

/// Renders the whole categories page
async fn get_categories(State(pool): State<SqlitePool>) -> Html<String> {
    let c = db::categories::get_all_categories(&pool).await.unwrap();
    let page = CatgeoriesPage {
        categories: c.into_iter().map(|c| CategoryRow { category: c }).collect(),
    };

    Html(page.render().unwrap())
}

async fn get_category(State(pool): State<SqlitePool>, Path(id): Path<i64>) -> Html<String> {
    let c = db::categories::get_category(&pool, id).await.unwrap();
    let row = CategoryRow { category: c };

    Html(row.render().unwrap())
}

async fn edit_category(State(pool): State<SqlitePool>, Path(id): Path<i64>) -> Html<String> {
    let c = db::categories::get_category(&pool, id).await.unwrap();
    let row = CategoryRowEdit { category: c };

    Html(row.render().unwrap())
}
/// Renders _just_ the created row and sends it to HTMX to work its majic
async fn create_category(
    State(pool): State<SqlitePool>,
    Form(new_category): Form<NewCategory>,
) -> impl IntoResponse {
    let id = db::categories::create_category(
        &pool,
        new_category.name.as_str(),
        new_category.hidden.unwrap_or(false),
        1,
    )
    .await
    .unwrap();

    let row = CategoryRow {
        category: db::categories::get_category(&pool, id).await.unwrap(),
    };
    Html(row.render().unwrap())
}

async fn update_category(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Form(category): Form<CategoryUpdate>,
) -> impl IntoResponse {
    db::categories::update_category(&pool, id, category.name, category.hidden.unwrap_or(false))
        .await
        .unwrap();
    let row = CategoryRow {
        category: db::categories::get_category(&pool, id).await.unwrap(),
    };
    Html(row.render().unwrap())
}

async fn delete_category(State(pool): State<SqlitePool>, Path(id): Path<i64>) -> impl IntoResponse {
    db::categories::delete_category(&pool, id).await.unwrap();
    StatusCode::OK
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
        .with_state(state)
}
