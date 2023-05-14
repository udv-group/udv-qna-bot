use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use db::Question;

#[derive(Deserialize)]
struct NewQuestion {
    name: String,
    hidden: bool,
    ordering: i64
}

async fn get_questions(State(pool): State<Pool>) -> Json<Vec<Question>> {
    db::questions::get_public_questions(&pool).await.unwrap()
}

async fn create_question(State(pool): State<Pool>) -> Json<Question> {
    !todo!()
}

async fn update_question(State(pool): State<Pool>) -> Json<Question> {
    !todo!()
}

async fn attach_file(State(pool): State<Pool>, question_id: Query<i64>) {
    !todo!()
}