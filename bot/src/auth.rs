use dotenv;
use sqlx::SqlitePool;
use teloxide::types::User;

pub async fn auth_user(conn: &SqlitePool, user: &User) -> anyhow::Result<bool> {
    if dotenv::var("USE_AUTH")
        .expect("Variable USE_AUTH should be set")
        .parse()
        .expect("Should be 'true' or 'false'")
    {
        // todo: return false only on RowNotFound
        db::users::get_user(conn, user.id.0.try_into().unwrap())
            .await
            .and_then(|user| Ok(user.active))
            .or(Ok(false))
    } else {
        if !db::users::get_user(conn, user.id.0.try_into().unwrap())
            .await
            .is_ok()
        {
            db::users::create_user(
                conn,
                user.id.0.try_into().unwrap(),
                user.username.as_ref().unwrap(),
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
