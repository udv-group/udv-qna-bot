use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashSet;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub is_admin: bool,
    pub active: bool,
}

pub async fn get_user(pool: &SqlitePool, id: i64) -> sqlx::Result<User> {
    sqlx::query_as!(
        User,
        r#"
        SELECT * FROM users WHERE users.id = ?1
        "#,
        id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_users(pool: &SqlitePool) -> sqlx::Result<Vec<User>> {
    sqlx::query_as!(
        User,
        r#"
        SELECT * FROM users
        "#,
    )
    .fetch_all(pool)
    .await
}
pub async fn create_user(
    pool: &SqlitePool,
    username: &str,
    first_name: &str,
    last_name: Option<&str>,
    is_admin: bool,
    active: bool,
) -> sqlx::Result<i64> {
    let mut conn = pool.acquire().await?;
    let id = sqlx::query!(
        r#"
        INSERT INTO users (username, first_name, last_name, is_admin, active) VALUES(?1, ?2, ?3, ?4, ?5)
        "#,
        username,
        first_name,
        last_name,
        is_admin,
        active
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}
pub async fn update_user(pool: &SqlitePool, user: User) -> sqlx::Result<()> {
    get_user(pool, user.id).await?;
    let mut conn = pool.acquire().await?;

    sqlx::query!(
        r#"
        UPDATE users SET username=?1, first_name=?2, last_name=?3, is_admin=?4, active=?5 WHERE users.id = ?6
        "#,
        user.username,
        user.first_name,
        user.last_name,
        user.is_admin,
        user.active,
        user.id
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}
pub async fn delete_user(pool: &SqlitePool, user_id: i64) -> sqlx::Result<()> {
    get_user(pool, user_id).await?;
    let mut conn = pool.acquire().await?;

    sqlx::query!(
        r#"
        DELETE FROM users WHERE users.id = ?1
        "#,
        user_id,
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

pub async fn import_users(pool: &SqlitePool, users: Vec<User>) -> sqlx::Result<()> {
    let existing_users = get_users(pool).await?;
    let existing_users_ids: HashSet<i64> = existing_users.iter().map(|c| c.id).collect();
    let new_users_ids: HashSet<i64> = users.iter().map(|c| c.id).collect();
    for user_id in existing_users_ids.difference(&new_users_ids) {
        delete_user(pool, *user_id).await?;
    }
    for user in users {
        if existing_users_ids.contains(&user.id) {
            update_user(pool, user).await?;
        } else {
            create_user(
                pool,
                &user.username.unwrap_or_else(|| "".to_string()),
                &user.first_name,
                user.last_name.as_deref(),
                user.is_admin,
                user.active,
            )
            .await?;
        }
    }
    Ok(())
}
