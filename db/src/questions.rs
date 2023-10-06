use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};

use crate::{Category, Reorder};

#[derive(Serialize, Deserialize, sqlx::FromRow)]
struct QuestionRowJoined {
    id: i64,
    category: Option<i64>,
    question: String,
    answer: String,
    attachments: String,
    hidden: bool,
    ordering: i64,
    category_id: i64,
    category_name: String,
    category_hidden: bool,
    category_ordering: i64,
}
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

#[derive(Debug)]
pub struct Question {
    pub id: i64,
    pub category: Option<Category>,
    pub question: String,
    pub answer: String,
    pub attachments: Vec<String>,
    pub hidden: bool,
    pub ordering: i64,
}

impl From<QuestionRowJoined> for Question {
    fn from(value: QuestionRowJoined) -> Self {
        Question {
            id: value.id,
            category: value.category.and_then(|_| {
                Some(Category {
                    id: value.category_id,
                    name: value.category_name,
                    hidden: value.category_hidden,
                    ordering: value.category_ordering,
                })
            }),
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
        QuestionRowJoined,
        r#"
        SELECT categories.id as category_id, categories.name as category_name, categories.hidden as category_hidden, categories.ordering as category_ordering,
         questions.id, questions.category, questions.question, questions.answer, questions.attachments, questions.hidden, questions.ordering 
        FROM questions JOIN categories on questions.category = categories.id WHERE categories.name = ?1 AND categories.hidden = FALSE AND questions.hidden = FALSE
        ORDER BY questions.ordering, questions.id DESC
        "#,
        category
    ).fetch_all(pool).await
    .map(|questions| questions.into_iter().map(|q| q.into()).collect())
}

pub async fn get_questions_by_category_id(
    pool: &SqlitePool,
    category_id: i64,
) -> sqlx::Result<Vec<Question>> {
    sqlx::query_as!(
        QuestionRowJoined,
        r#"
        SELECT categories.id as category_id, categories.name as category_name, categories.hidden as category_hidden, categories.ordering as category_ordering,
            questions.*  
        FROM questions JOIN categories on questions.category = categories.id 
        WHERE questions.category = ?1 
        ORDER BY questions.ordering, questions.id DESC
        "#,
        category_id
    ).fetch_all(pool).await
    .map(|questions| questions.into_iter().map(|q| q.into()).collect())
}
pub async fn get_question_by_category_name(
    pool: &SqlitePool,
    question: &str,
    category: &str,
) -> sqlx::Result<Question> {
    sqlx::query_as!(
        QuestionRowJoined,
        r#"
        SELECT categories.id as category_id, categories.name as category_name, categories.hidden as category_hidden, categories.ordering as category_ordering,
            questions.* 
        FROM questions JOIN categories on questions.category = categories.id WHERE categories.name = ?1 AND questions.question = ?2
        "#,
        category,
        question,
    )
    .fetch_one(pool)
    .await.map(|x| x.into())
}

pub async fn get_question_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Question> {
    let question_row = sqlx::query_as!(
        QuestionRow,
        r#"
        SELECT * FROM questions WHERE questions.id = ?1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;

    let category = if let Some(c) = question_row.category {
        crate::categories::get_category(&pool, c).await.ok()
    } else {
        None
    };

    Ok(Question {
        id: question_row.id,
        category,
        question: question_row.question,
        answer: question_row.answer,
        attachments: serde_json::from_str(&question_row.attachments).unwrap(),
        hidden: question_row.hidden,
        ordering: question_row.ordering,
    })
}

pub async fn get_all_questions(pool: &SqlitePool) -> sqlx::Result<Vec<Question>> {
    let questions_rows = sqlx::query_as!(
        QuestionRow,
        r#"
        SELECT * FROM questions ORDER BY questions.ordering, questions.id DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    let categories: HashMap<i64, Category> = crate::categories::get_all_categories(&pool)
        .await?
        .into_iter()
        .map(|c| (c.id, c))
        .collect();
    Ok(questions_rows
        .into_iter()
        .map(|q| Question {
            id: q.id,
            category: q.category.and_then(|c| categories.get(&c).cloned()),
            question: q.question,
            answer: q.answer,
            attachments: serde_json::from_str(&q.attachments).unwrap(),
            hidden: q.hidden,
            ordering: q.ordering,
        })
        .collect())
}

pub async fn get_public_questions(pool: &SqlitePool) -> sqlx::Result<Vec<Question>> {
    let questions_rows = sqlx::query_as!(
        QuestionRow,
        r#"
        SELECT * FROM questions WHERE hidden = FALSE ORDER BY ordering, questions.id DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    let categories: HashMap<i64, Category> = crate::categories::get_all_categories(&pool)
        .await?
        .into_iter()
        .map(|c| (c.id, c))
        .collect();
    Ok(questions_rows
        .into_iter()
        .map(|q| Question {
            id: q.id,
            category: categories.get(&q.id).cloned(),
            question: q.question,
            answer: q.answer,
            attachments: serde_json::from_str(&q.attachments).unwrap(),
            hidden: q.hidden,
            ordering: q.ordering,
        })
        .collect())
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

pub async fn reorder_questions(pool: &SqlitePool, questions: Vec<Reorder>) -> sqlx::Result<()> {
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
                question.category.map(|c| c.id),
                question.question,
                question.answer,
                question.attachments.iter().map(|x| x.as_str()).collect(),
                question.hidden,
            )
            .await?;
        } else {
            create_question(
                pool,
                question.question.as_str(),
                question.answer.as_str(),
                question.category.map(|c| c.id),
                question.attachments.iter().map(|x| x.as_str()).collect(),
                question.hidden,
                question.ordering,
            )
            .await?;
        }
    }
    Ok(())
}
