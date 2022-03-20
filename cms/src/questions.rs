use crate::error_handlers::{EmptyResult, JsonResult};
use db::questions::Question;
use rocket::serde::{json::Json, Deserialize};
use rocket::{Route, State};
use sqlx::SqlitePool;

#[derive(Deserialize)]
struct NewQuestion {
    category: Option<i64>,
    question: String,
    answer: String,
}

#[get("/questions")]
async fn get_questions(pool: &State<SqlitePool>) -> JsonResult<Vec<Question>> {
    let question = db::questions::get_questions(pool).await?;
    Ok(Json(question))
}

#[post("/questions", format = "json", data = "<question>")]
async fn create_question(
    question: Json<NewQuestion>,
    pool: &State<SqlitePool>,
) -> JsonResult<Question> {
    let question = question.into_inner();
    db::questions::create_question(
        pool,
        question.question.as_str(),
        question.answer.as_str(),
        question.category,
    )
    .await?;
    let new_question = db::questions::get_question(pool, question.question.as_str()).await?;
    Ok(Json(new_question))
}
#[patch("/questions", format = "json", data = "<question>")]
async fn update_question(
    question: Json<Question>,
    pool: &State<SqlitePool>,
) -> JsonResult<Question> {
    let question_inner = question.into_inner();
    let question = question_inner.question.clone();
    db::questions::update_question(pool, question_inner).await?;
    let question = db::questions::get_question(pool, question.as_str()).await?;
    Ok(Json(question))
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
