use sqlx::SqlitePool;
use teloxide::types::User;

use crate::db::queries::users::{create_user, get_user};

pub async fn auth_user(conn: &SqlitePool, user: &User) -> anyhow::Result<bool> {
    if dotenv::var("USE_AUTH")
        .expect("Variable USE_AUTH should be set")
        .parse()
        .expect("Should be 'true' or 'false'")
    {
        // todo: return false only on RowNotFound
        get_user(conn, user.id.0.try_into().unwrap())
            .await
            .map(|user| user.active)
            .or(Ok(false))
    } else {
        if get_user(conn, user.id.0.try_into().unwrap()).await.is_err() {
            create_user(
                conn,
                user.id.0.try_into().unwrap(),
                user.username.as_deref(),
                &user.first_name,
                user.last_name.as_deref(),
                false,
                false,
            )
            .await?;
        }
        Ok(true)
    }
}
