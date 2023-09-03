use crate::AppState;
use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{delete, get, post},
    Json, Router,
};
use db::Category;
use serde::{Deserialize, Deserializer};
use sqlx::{Pool, Sqlite, SqlitePool};

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

#[derive(Template)]
#[template(path = "categories/categories_reordering.html", escape = "none")]
struct CatgeoriesPageReordering {
    categories: Vec<Category>,
}

/// Renders the whole categories page
async fn get_categories(State(pool): State<SqlitePool>) -> Html<String> {
    let categories = db::categories::get_all_categories(&pool).await.unwrap();
    let page = CatgeoriesPage {
        categories: categories
            .into_iter()
            .map(|c| CategoryRow { category: c })
            .collect(),
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

    let row = CategoryRow {
        category: db::categories::get_category(&pool, id).await.unwrap(),
    };
    Html(row.render().unwrap())
}

async fn update_category(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(category): Json<CategoryUpdate>,
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

async fn render_reordering_page(State(pool): State<SqlitePool>) -> impl IntoResponse {
    let categories = db::categories::get_all_categories(&pool).await.unwrap();
    let page = CatgeoriesPageReordering { categories };

    Html(page.render().unwrap())
}

#[derive(Deserialize)]
struct OrderingBody {
    row_id: Vec<Ordering>,
}

#[derive(Deserialize)]
#[serde(try_from = "String")]
struct Ordering(pub i64);
impl TryFrom<String> for Ordering {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.parse::<i64>() {
            Ok(v) => Ok(Ordering(v)),
            Err(_) => Err("???".to_owned()),
        }
    }
}

async fn reorder(
    State(pool): State<SqlitePool>,
    Json(body): Json<OrderingBody>,
) -> impl IntoResponse {
    let ordering: Vec<db::categories::CategoryReorder> = body
        .row_id
        .into_iter()
        .enumerate()
        .map(|(n, v)| db::categories::CategoryReorder {
            id: v.0,
            ordering: n as i64,
        })
        .collect();

    db::categories::reorder_categories(&pool, ordering)
        .await
        .unwrap();

    let categories = db::categories::get_all_categories(&pool).await.unwrap();
    let page = CatgeoriesPage {
        categories: categories
            .into_iter()
            .map(|c| CategoryRow { category: c })
            .collect(),
    };

    Html(page.render().unwrap())
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
