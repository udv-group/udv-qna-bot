use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Serialize, Deserialize)]
pub struct Question {
    pub id: i64,
    pub category: Option<i64>,
    pub question: String,
    pub answer: String,
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

pub async fn get_questions(pool: &SqlitePool) -> sqlx::Result<Vec<Question>> {
    sqlx::query_as!(
        Question,
        r#"
        SELECT * FROM questions ORDER BY id 
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn create_question(
    pool: &SqlitePool,
    question: &str,
    answer: &str,
    category: Option<i64>,
) -> anyhow::Result<i64> {
    let mut conn = pool.acquire().await?;

    let id = sqlx::query!(
        r#"
INSERT INTO questions (category, question, answer) VALUES (?1, ?2, ?3)
        "#,
        category,
        question,
        answer
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}

pub async fn update_question(pool: &SqlitePool, question: Question) -> anyhow::Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query!(
        r#"
        UPDATE questions SET category=?1, question=?2, answer=?3 WHERE questions.id = ?4
        "#,
        question.category,
        question.question,
        question.answer,
        question.id
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}
