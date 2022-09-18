use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashSet;

#[derive(Serialize, Deserialize)]
pub struct Question {
    pub id: i64,
    pub category: Option<i64>,
    pub question: String,
    pub answer: String,
    pub attachment: Option<String>,
    pub hidden: bool,
}

pub async fn get_public_questions_for_public_category(
    pool: &SqlitePool,
    category: &str,
) -> sqlx::Result<Vec<Question>> {
    sqlx::query_as!(
        Question,
        r#"
        SELECT questions.id, questions.category, questions.question, questions.answer, questions.attachment, questions.hidden 
        FROM questions JOIN categories on questions.category = categories.id WHERE categories.name = ?1 AND categories.hidden = FALSE AND questions.hidden = FALSE
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

pub async fn get_question_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Question> {
    sqlx::query_as!(
        Question,
        r#"
        SELECT * FROM questions WHERE questions.id = ?1
        "#,
        id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_all_questions(pool: &SqlitePool) -> sqlx::Result<Vec<Question>> {
    sqlx::query_as!(
        Question,
        r#"
        SELECT * FROM questions ORDER BY id 
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn get_public_questions(pool: &SqlitePool) -> sqlx::Result<Vec<Question>> {
    sqlx::query_as!(
        Question,
        r#"
        SELECT * FROM questions WHERE hidden = FALSE ORDER BY id
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
    attachment: Option<&str>,
    hidden: bool,
) -> sqlx::Result<i64> {
    let mut conn = pool.acquire().await?;

    let id = sqlx::query!(
        r#"
        INSERT INTO questions (category, question, answer, attachment, hidden) VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        category,
        question,
        answer,
        attachment,
        hidden
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}

pub async fn update_question(pool: &SqlitePool, question: Question) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query!(
        r#"
        UPDATE questions SET category=?1, question=?2, answer=?3, attachment=?4, hidden=?5 WHERE questions.id = ?6
        "#,
        question.category,
        question.question,
        question.answer,
        question.attachment,
        question.hidden,
        question.id
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

pub async fn delete_question(pool: &SqlitePool, question_id: i64) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query!(
        r#"
        DELETE FROM questions WHERE questions.id = ?1
        "#,
        question_id,
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

pub async fn import_questions(pool: &SqlitePool, questions: Vec<Question>) -> sqlx::Result<()> {
    let existing_questions = get_all_questions(pool).await?;
    let existing_questions_ids: HashSet<i64> = existing_questions.iter().map(|q| q.id).collect();
    let new_questions_ids: HashSet<i64> = questions.iter().map(|q| q.id).collect();
    for question_id in existing_questions_ids.difference(&new_questions_ids) {
        delete_question(pool, *question_id).await?;
    }
    for question in questions {
        if existing_questions_ids.contains(&question.id) {
            update_question(pool, question).await?;
        } else {
            create_question(
                pool,
                question.question.as_str(),
                question.answer.as_str(),
                question.category,
                question.attachment.as_deref(),
                question.hidden,
            )
            .await?;
        }
    }
    Ok(())
}
