use crate::error_handlers::{EmptyResult, JsonResult};
use db::users::User;
use rocket::serde::{json::Json, Deserialize};
use rocket::{Route, State};
use sqlx::SqlitePool;

#[derive(Deserialize)]
struct NewUser {
    id: i64,
    username: String,
    first_name: String,
    last_name: Option<String>,
    active: bool,
    is_admin: bool,
}

#[get("/users")]
async fn get_users(pool: &State<SqlitePool>) -> JsonResult<Vec<User>> {
    let users = db::users::get_users(pool).await?;
    Ok(Json(users))
}

#[post("/users", format = "json", data = "<user>")]
async fn create_user(user: Json<NewUser>, pool: &State<SqlitePool>) -> JsonResult<User> {
    let user = user.into_inner();
    let user_id = db::users::create_user(
        pool,
        user.id,
        &user.username,
        &user.first_name,
        user.last_name.as_deref(),
        user.is_admin,
        user.active,
    )
    .await?;
    let new_user = db::users::get_user(pool, user_id).await?;
    Ok(Json(new_user))
}

#[patch("/users", format = "json", data = "<user>")]
async fn update_user(user: Json<User>, pool: &State<SqlitePool>) -> JsonResult<User> {
    let user_inner = user.into_inner();
    let user_id = user_inner.id;
    db::users::update_user(pool, user_inner).await?;
    let user = db::users::get_user(pool, user_id).await?;
    Ok(Json(user))
}

#[delete("/users/<user_id>")]
async fn delete_user(user_id: i64, pool: &State<SqlitePool>) -> EmptyResult {
    db::users::delete_user(pool, user_id).await?;
    Ok(())
}

pub fn routes() -> Vec<Route> {
    routes![get_users, create_user, update_user, delete_user]
}
