pub mod categories;
pub mod questions;
pub mod users;
use sqlx::sqlite::SqlitePool;

use sqlx::Error;

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
