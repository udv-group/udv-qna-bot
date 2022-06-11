use dotenv;
use sqlx::SqlitePool;

pub async fn auth_user(conn: &SqlitePool, id: i64) -> bool {
    if dotenv::var("USE_AUTH")
        .expect("Variable USE_AUTH should be set")
        .parse()
        .expect("Should be 'true' or 'false'")
    {
        db::users::get_user(conn, id).await.is_ok()
    } else {
        true
    }
}
