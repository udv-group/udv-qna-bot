use sqlx::SqlitePool;

pub struct User {
    pub id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
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
    username: String,
    first_name: String,
    last_name: Option<String>,
) -> sqlx::Result<i64> {
    let mut conn = pool.acquire().await?;
    let id = sqlx::query!(
        r#"
        INSERT INTO users (username, first_name, last_name) VALUES(?1, ?2, ?3)
        "#,
        username,
        first_name,
        last_name
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}
pub async fn update_user(pool: &SqlitePool, user: User) -> anyhow::Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query!(
        r#"
        UPDATE users SET username=?1, first_name=?2, last_name=?3 WHERE users.id = ?4
        "#,
        user.username,
        user.first_name,
        user.last_name,
        user.id
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}
