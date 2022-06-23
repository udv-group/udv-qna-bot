use crate::error_handlers::{EmptyResult};
use db::questions::{Question};
use rocket::response::Redirect;

use rocket::{Route, State};
use rocket_dyn_templates::{context, Template};
use sqlx::SqlitePool;
use std::path::PathBuf;

use rocket::form::Form;
use rocket::fs::TempFile;

#[derive(FromForm)]
struct QuestionUpdate<'r> {
    id: i64,
    category: Option<i64>,
    question: String,
    answer: String,
    attachment: Option<TempFile<'r>>,
}

#[derive(FromForm)]
struct NewQuestion<'r> {
    category: Option<i64>,
    question: String,
    answer: String,
    attachment: Option<TempFile<'r>>,
}

#[get("/questions")]
async fn get_questions(pool: &State<SqlitePool>) -> Template {
    let questions = db::questions::get_questions(pool).await.unwrap();
    Template::render(
        "questions",
        context! {
            questions: questions,
            title: "Questions"
        },
    )
}
#[post("/questions", data = "<question>")]
async fn update_question(question: Form<QuestionUpdate<'_>>, pool: &State<SqlitePool>) -> Redirect {
    let static_dir =
        PathBuf::from(dotenv::var("STATIC_DIR").expect("Variable STATIC_DIR should be set"));
    let mut question = question.into_inner();
    let old_question = db::questions::get_question(pool, question.question.as_str())
        .await
        .unwrap();
    let mut filename: Option<String> = old_question.attachment; // FIXME: add ability to remove attached files
                                                                // TODO: don't send file if it was not selected
    if let Some(file) = &mut question.attachment {
        if file.len() > 0 {
            if let Some(name) = file.name() {
                let name = name.to_owned();
                file.persist_to(static_dir.join(&name)).await.unwrap();
                filename = Some(name);
            }
        }
    }
    let question = Question {
        id: question.id,
        category: question.category,
        answer: question.answer,
        question: question.question,
        attachment: filename,
    };
    db::questions::update_question(pool, question)
        .await
        .unwrap();
    Redirect::to(uri!(get_questions))
}

#[post("/questions/new", data = "<question>")]
async fn create_question(
    mut question: Form<NewQuestion<'_>>,
    pool: &State<SqlitePool>,
) -> Redirect {
    let static_dir =
        PathBuf::from(dotenv::var("STATIC_DIR").expect("Variable STATIC_DIR should be set"));
    let mut filename: Option<String> = None;
    // TODO: don't send file if it was not selected
    if let Some(file) = &mut question.attachment {
        if file.len() > 0 {
            if let Some(name) = file.name() {
                let name = name.to_owned();
                file.persist_to(static_dir.join(&name)).await.unwrap();
                filename = Some(name);
            }
        }
    }
    db::questions::create_question(
        pool,
        question.question.as_str(),
        question.answer.as_str(),
        question.category,
        filename.as_deref(),
    )
    .await
    .unwrap();
    Redirect::to(uri!(get_questions))
}

#[delete("/questions/<question_id>")]
async fn delete_question(question_id: i64, pool: &State<SqlitePool>) -> EmptyResult {
    db::questions::delete_question(pool, question_id).await?;
    Ok(())
}
pub fn routes() -> Vec<Route> {
    routes![
        get_questions,
        create_question,
        update_question,
        delete_question
    ]
}
