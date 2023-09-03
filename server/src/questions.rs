use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use db::Question;
use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Deserialize)]
struct NewQuestion {
    name: String,
    hidden: bool,
    ordering: i64
}

async fn get_questions(State(pool): State<SqlitePool>) -> Json<Vec<Question>> {
    todo!()
}

async fn create_question(State(pool): State<SqlitePool>) -> Json<Question> {
    todo!()
}

async fn update_question(State(pool): State<SqlitePool>) -> Json<Question> {
    todo!()
}

async fn attach_file(State(pool): State<SqlitePool>, question_id: Query<i64>) {
    todo!()
}