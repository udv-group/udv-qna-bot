use crate::error_handlers::{EmptyResult, JsonResult};
use db::questions::Question;
use rocket::response::Redirect;
use rocket::serde::{json::Json, Deserialize};
use rocket::{Route, State};
use rocket_dyn_templates::{context, tera::Tera, Template};
use sqlx::SqlitePool;

use rocket::form::Form;
use rocket::fs::TempFile;

#[derive(Deserialize)]
struct NewQuestion {
    category: Option<i64>,
    question: String,
    answer: String,
    attachment: Option<String>,
}

#[derive(FromForm)]
struct QuestionUpload<'r> {
    id: i64,
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
#[post("/questions", data = "<upload>")]
fn update_question(upload: Form<QuestionUpload<'_>>) -> Redirect {
    println!("{}", upload.question);
    println!("{}", upload.answer);
    println!("{:#?}", upload.attachment);
    Redirect::to(uri!(get_questions))
}
// #[post("/questions", format = "json", data = "<question>")]
// async fn create_question(
//     question: Json<NewQuestion>,
//     pool: &State<SqlitePool>,
// ) -> JsonResult<Question> {
//     let question = question.into_inner();
//     db::questions::create_question(
//         pool,
//         question.question.as_str(),
//         question.answer.as_str(),
//         question.category,
//         question.attachment.as_deref(),
//     )
//     .await?;
//     let new_question = db::questions::get_question(pool, question.question.as_str()).await?;
//     Ok(Json(new_question))
// }
// #[patch("/questions", format = "json", data = "<question>")]
// async fn update_question(
//     question: Json<Question>,
//     pool: &State<SqlitePool>,
// ) -> JsonResult<Question> {
//     let question_inner = question.into_inner();
//     let question = question_inner.question.clone();
//     db::questions::update_question(pool, question_inner).await?;
//     let question = db::questions::get_question(pool, question.as_str()).await?;
//     Ok(Json(question))
// }
#[delete("/questions/<question_id>")]
async fn delete_question(question_id: i64, pool: &State<SqlitePool>) -> EmptyResult {
    db::questions::delete_question(pool, question_id).await?;
    Ok(())
}
pub fn routes() -> Vec<Route> {
    routes![
        get_questions,
        // create_question,
        update_question,
        delete_question
    ]
}
