use sqlx::SqlitePool;

pub async fn auth_user(conn: &SqlitePool, id: i64) -> bool {
    db::users::get_user(conn, id).await.is_ok()
}
