use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashSet;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
struct QuestionRow {
    id: i64,
    category: Option<i64>,
    question: String,
    answer: String,
    attachments: String,
    hidden: bool,
    ordering: i64,
}

pub struct Question {
    pub id: i64,
    pub category: Option<i64>,
    pub question: String,
    pub answer: String,
    pub attachments: Vec<String>,
    pub hidden: bool,
    pub ordering: i64,
}

impl From<QuestionRow> for Question {
    fn from(value: QuestionRow) -> Self {
        Question {
            id: value.id,
            category: value.category,
            question: value.question,
            answer: value.answer,
            attachments: serde_json::from_str(&value.attachments).unwrap(),
            hidden: value.hidden,
            ordering: value.ordering,
        }
    }
}

pub async fn get_public_questions_for_public_category(
    pool: &SqlitePool,
    category: &str,
) -> sqlx::Result<Vec<Question>> {
    sqlx::query_as!(
        QuestionRow,
        r#"
        SELECT questions.id, questions.category, questions.question, questions.answer, questions.attachments, questions.hidden, questions.ordering 
        FROM questions JOIN categories on questions.category = categories.id WHERE categories.name = ?1 AND categories.hidden = FALSE AND questions.hidden = FALSE
        ORDER BY questions.ordering
        "#,
        category
    ).fetch_all(pool).await
    .map(|questions| questions.into_iter().map(|q| q.into()).collect())
}

pub async fn get_question(
    pool: &SqlitePool,
    question: &str,
    category: &str,
) -> sqlx::Result<Question> {
    sqlx::query_as!(
        QuestionRow,
        r#"
        SELECT questions.id, questions.category, questions.question, questions.answer, questions.attachments, questions.hidden, questions.ordering 
        FROM questions JOIN categories on questions.category = categories.id WHERE categories.name = ?1 AND questions.question = ?2
        "#,
        category,
        question,
    )
    .fetch_one(pool)
    .await.map(|x| x.into())
}

pub async fn get_question_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Question> {
    sqlx::query_as!(
        QuestionRow,
        r#"
        SELECT * FROM questions WHERE questions.id = ?1
        "#,
        id
    )
    .fetch_one(pool)
    .await
    .map(|q| q.into())
}

pub async fn get_all_questions(pool: &SqlitePool) -> sqlx::Result<Vec<Question>> {
    sqlx::query_as!(
        QuestionRow,
        r#"
        SELECT * FROM questions ORDER BY ordering
        "#,
    )
    .fetch_all(pool)
    .await
    .map(|questions| questions.into_iter().map(|q| q.into()).collect())
}

pub async fn get_public_questions(pool: &SqlitePool) -> sqlx::Result<Vec<Question>> {
    sqlx::query_as!(
        QuestionRow,
        r#"
        SELECT * FROM questions WHERE hidden = FALSE ORDER BY ordering
        "#,
    )
    .fetch_all(pool)
    .await
    .map(|questions| questions.into_iter().map(|q| q.into()).collect())
}

pub async fn create_question(
    pool: &SqlitePool,
    question: &str,
    answer: &str,
    category: Option<i64>,
    attachments: Vec<&str>,
    hidden: bool,
    ordering: i64,
) -> sqlx::Result<i64> {
    let mut conn = pool.acquire().await?;
    let att = serde_json::to_string(&attachments).unwrap();
    let id = sqlx::query!(
        r#"
        INSERT INTO questions (category, question, answer, attachments, hidden, ordering) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        category,
        question,
        answer,
        att,
        hidden,
        ordering,
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}

pub async fn update_question(
    pool: &SqlitePool,
    id: i64,
    category: Option<i64>,
    question: String,
    answer: String,
    attachments: Vec<&str>,
    hidden: bool,
) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;
    let att = serde_json::to_string(&attachments).unwrap();
    sqlx::query!(
        r#"
        UPDATE questions SET category=?1, question=?2, answer=?3, attachments=?4, hidden=?5 WHERE questions.id = ?6
        "#,
        category,
        question,
        answer,
        att,
        hidden,
        id,
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

pub async fn reorder_questions(pool: &SqlitePool, questions: Vec<Question>) -> sqlx::Result<()> {
    let mut transaction = pool.begin().await?;
    for question in questions {
        sqlx::query!(
            r#"
            UPDATE questions SET ordering=?1 WHERE questions.id = ?2
            "#,
            question.ordering,
            question.id,
        )
        .execute(&mut transaction)
        .await?;
    }
    transaction.commit().await?;
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
            update_question(
                pool,
                question.id,
                question.category,
                question.question,
                question.answer,
                question
                    .attachments
                    .iter()
                    .map(|x| x.as_str())
                    .collect(),
                question.hidden,
            )
            .await?;
        } else {
            create_question(
                pool,
                question.question.as_str(),
                question.answer.as_str(),
                question.category,
                question
                    .attachments
                    .iter()
                    .map(|x| x.as_str())
                    .collect(),
                question.hidden,
                question.ordering,
            )
            .await?;
        }
    }
    Ok(())
}
