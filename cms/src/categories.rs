use db::categories::Category;
use rocket::form::{Form, FromForm};
use rocket::response::Redirect;
use rocket::{Route, State};
use rocket_dyn_templates::{context, Template};
use sqlx::SqlitePool;

#[derive(FromForm)]
struct CategoryUpdate {
    id: i64,
    name: String,
}

#[derive(FromForm)]
struct NewCategory {
    name: String,
}

#[get("/categories")]
async fn get_categories(pool: &State<SqlitePool>) -> Template {
    let categories = db::categories::get_categories(pool).await.unwrap();
    Template::render(
        "categories",
        context! {
            categories: categories,
            title: "Categories"
        },
    )
}

#[post("/categories/new", data = "<category>")]
async fn create_category(category: Form<NewCategory>, pool: &State<SqlitePool>) -> Redirect {
    let category = category.into_inner();
    db::categories::create_category(pool, category.name.as_str())
        .await
        .unwrap();
    Redirect::to(uri!(get_categories))
}
#[post("/categories", data = "<category>")]
async fn update_category(category: Form<CategoryUpdate>, pool: &State<SqlitePool>) -> Redirect {
    let category = category.into_inner();
    let category_update = Category {
        id: category.id,
        name: category.name,
    };
    db::categories::update_category(pool, category_update)
        .await
        .unwrap();
    Redirect::to(uri!(get_categories))
}
#[delete("/categories/<category_id>")]
async fn delete_category(category_id: i64, pool: &State<SqlitePool>) -> Redirect {
    db::categories::delete_category(pool, category_id)
        .await
        .unwrap();
    Redirect::to(uri!(get_categories))
}
pub fn routes() -> Vec<Route> {
    routes![
        get_categories,
        create_category,
        update_category,
        delete_category
    ]
}
