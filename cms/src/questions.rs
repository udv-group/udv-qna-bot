use db::questions::Question;
use rocket::response::Redirect;

use rocket::{Route, State};
use rocket_dyn_templates::{context, Template};
use sqlx::SqlitePool;
use std::path::PathBuf;

use rocket::form::Form;
use rocket::fs::TempFile;

use serde::Serialize;

#[derive(FromForm)]
struct QuestionUpdate<'r> {
    id: i64,
    category: Option<i64>,
    question: String,
    answer: String,
    attachment: Option<TempFile<'r>>,
    hidden: bool,
}

#[derive(FromForm)]
struct NewQuestion<'r> {
    category: Option<i64>,
    question: String,
    answer: String,
    attachment: TempFile<'r>,
    hidden: bool,
}

#[derive(Serialize)]
struct ShowQuestion {
    id: i64,
    category: Option<i64>,
    question: String,
    answer: String,
    attachment: Option<String>,
    hidden: bool,
    answer_trunk: String,
}

fn find_nex_char_boundary(index: usize, string: &str) -> usize {
    let mut index = index;
    while !string.is_char_boundary(index) && index < string.len() {
        index += 1;
    }
    index
}

#[get("/questions")]
async fn get_questions(pool: &State<SqlitePool>) -> Template {
    let questions: Vec<ShowQuestion> = db::questions::get_all_questions(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|q| {
            let mut trunk = q.answer.clone();
            if trunk.len() > 29 {
                trunk.truncate(find_nex_char_boundary(25, &trunk));
                trunk.push_str(" ...");
            }
            ShowQuestion {
                id: q.id,
                category: q.category,
                question: q.question,
                answer: q.answer,
                attachment: q.attachment,
                hidden: q.hidden,
                answer_trunk: trunk,
            }
        })
        .collect();
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
    let old_question = db::questions::get_question_by_id(pool, question.id)
        .await
        .unwrap();
    let mut filename: Option<String> = old_question.attachment; // FIXME: add ability to remove attached files
                                                                // TODO: don't send file if it was not selected
    if let Some(file) = &mut question.attachment {
        if file.len() > 0 {
            if let Some(name) = file.name() {
                let name = name.to_owned();
                file.move_copy_to(static_dir.join(&name)).await.unwrap();
                filename = Some(name);
            }
        } else {
            if let Some(old_f) = filename {
                let filepath = static_dir.join(old_f);
                if filepath.exists() {
                    // TODO: do not delete it if other questions are linked to it
                    std::fs::remove_file(filepath).expect("File exists");
                }
            }
            filename = None;
        }
    }
    let question = Question {
        id: question.id,
        category: question.category,
        answer: question.answer,
        question: question.question,
        hidden: question.hidden,
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
    let file = &mut question.attachment;

    if file.len() > 0 {
        if let Some(name) = file.name() {
            let name = name.to_owned();
            file.move_copy_to(static_dir.join(&name)).await.unwrap();
            filename = Some(name);
        }
    }

    db::questions::create_question(
        pool,
        question.question.as_str(),
        question.answer.as_str(),
        question.category,
        filename.as_deref(),
        question.hidden,
    )
    .await
    .unwrap();
    Redirect::to(uri!(get_questions))
}

#[delete("/questions/<question_id>")]
async fn delete_question(question_id: i64, pool: &State<SqlitePool>) -> Redirect {
    db::questions::delete_question(pool, question_id)
        .await
        .unwrap();
    Redirect::to(uri!(get_questions))
}

pub fn routes() -> Vec<Route> {
    routes![
        get_questions,
        create_question,
        update_question,
        delete_question
    ]
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_find_boundary() {
        let s = "some string";
        assert_eq!(find_nex_char_boundary(3, s), 3)
    }

    #[test]
    fn test_find_boundary_cyrillic() {
        let s = "немного текста";
        assert_eq!(find_nex_char_boundary(3, s), 4)
    }

    #[test]
    fn test_find_boundary_end_of_text() {
        let s = "short";
        assert_eq!(find_nex_char_boundary(10, s), 10)
    }
}
