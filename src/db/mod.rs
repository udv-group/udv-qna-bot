pub mod queries;

use sqlx::sqlite::SqlitePool;

pub use queries::categories::Category;
pub use queries::questions::Question;
pub use queries::users::User;

use sqlx::Error;

pub struct Reorder {
    pub id: i64,
    pub ordering: i64,
}

pub async fn establish_connection(path: &str) -> Result<SqlitePool, Error> {
    SqlitePool::connect(format!("sqlite:{}", path).as_str()).await
}

pub async fn run_migrations() -> Result<(), Error> {
    dotenv::dotenv().ok();
    let path = dotenv::var("DB_PATH").expect("DB_PATH must be set");
    let pool = establish_connection(&path).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {}
