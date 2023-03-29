use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Json, Router,
};
use db::User;
use sqlx::{Pool, Sqlite, SqlitePool};

async fn get_users(State(pool): State<SqlitePool>) -> Json<Vec<User>> {
    Json(db::users::get_users(&pool).await.unwrap())
}

async fn create_user(State(pool): State<SqlitePool>, Json(user): Json<User>) -> Json<User> {
    let user_id = db::users::create_user(
        &pool,
        user.id,
        user.username.as_deref(),
        &user.first_name,
        user.last_name.as_deref(),
        user.is_admin,
        user.active,
    )
    .await
    .unwrap();
    let new_user = db::users::get_user(&pool, user_id).await.unwrap();
    Json(new_user)
}

async fn update_user(State(pool): State<SqlitePool>, Json(user): Json<User>) -> Json<User> {
    let user_id = user.id;
    db::users::update_user(&pool, user).await.unwrap();
    let updated_user = db::users::get_user(&pool, user_id).await.unwrap();
    Json(updated_user)
}

async fn delete_user(
    State(pool): State<SqlitePool>,
    Path(user_id): Path<i64>,
) -> impl IntoResponse {
    db::users::delete_user(&pool, user_id).await.unwrap();
    StatusCode::OK
}

pub fn users_router(pool: Pool<Sqlite>) -> Router {
    Router::new()
        .route("/users", get(get_users).post(create_user).put(update_user))
        .route("/users/:id", delete(delete_user))
        .with_state(pool)
}
