pub mod models;

use sqlx::sqlite::SqlitePool;

extern crate dotenv;

use crate::models::*;
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

//todo: return Result
pub async fn create_category(pool: &SqlitePool, name: &str) -> anyhow::Result<i64> {
    let mut conn = pool.acquire().await?;

    let id = sqlx::query!(
        r#"
INSERT INTO categories (name) VALUES (?1)
        "#,
        name
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}

pub async fn get_categories(pool: &SqlitePool) -> sqlx::Result<Vec<Category>> {
    sqlx::query_as!(
        Category,
        r#"
SELECT id, name
FROM categories
ORDER BY id
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn get_questions_by_category(
    pool: &SqlitePool,
    category: &str,
) -> sqlx::Result<Vec<Question>> {
    sqlx::query_as!(
        Question,
        r#"
        SELECT questions.id, questions.category, questions.question, questions.answer FROM questions JOIN categories on questions.category = categories.id WHERE categories.name = ?1
        "#,
        category
    ).fetch_all(pool).await
}

pub async fn get_question(pool: &SqlitePool, question: &str) -> sqlx::Result<Question> {
    sqlx::query_as!(
        Question,
        r#"
        SELECT * FROM questions WHERE questions.question = ?1
        "#,
        question
    )
    .fetch_one(pool)
    .await
}

#[cfg(test)]
mod tests {}
