use crate::error_handlers::{EmptyResult, JsonResult};
use db::categories::Category;
use rocket::serde::{json::Json, Deserialize};
use rocket::{Route, State};
use sqlx::SqlitePool;

#[derive(Deserialize)]
struct NewCategory {
    name: String,
}

#[get("/categories")]
async fn get_categories(pool: &State<SqlitePool>) -> JsonResult<Vec<Category>> {
    let categories = db::categories::get_categories(pool).await?;
    Ok(Json(categories))
}

#[post("/categories", format = "json", data = "<category>")]
async fn create_category(
    category: Json<NewCategory>,
    pool: &State<SqlitePool>,
) -> JsonResult<Category> {
    let category = category.into_inner();
    db::categories::create_category(pool, category.name.as_str()).await?;
    let new_category = db::categories::get_category(pool, category.category.as_str()).await?;
    Ok(Json(new_category))
}
#[patch("/categories", format = "json", data = "<category>")]
async fn update_category(
    category: Json<Category>,
    pool: &State<SqlitePool>,
) -> JsonResult<Category> {
    let category_inner = category.into_inner();
    let category_id = category_inner.id;
    db::categories::update_category(pool, category_inner).await?;
    let category = db::categories::get_category(pool, category_id).await?;
    Ok(Json(category))
}
#[delete("/categories/<category_id>")]
async fn delete_category(category_id: i64, pool: &State<SqlitePool>) -> EmptyResult {
    db::categories::delete_category(pool, category_id).await?;
    Ok(())
}
pub fn routes() -> Vec<Route> {
    routes![
        get_categories,
        create_category,
        update_category,
        delete_category
    ]
}
