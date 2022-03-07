pub mod categories;
pub mod questions;
pub mod users;

use sqlx::sqlite::SqlitePool;

extern crate dotenv;

use dotenv::dotenv;
use sqlx::Error;

pub async fn establish_connection() -> Result<SqlitePool, Error> {
    dotenv().ok();
    let database_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqlitePool::connect(&database_url).await
}

pub async fn run_migrations() -> Result<(), Error> {
    let pool = establish_connection().await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {}
