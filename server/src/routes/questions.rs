use std::path::PathBuf;

use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    async_trait,
    body::Bytes,
    extract::{Path, State},
    http::HeaderMap,
    routing::get,
    Router,
};
use axum_typed_multipart::{
    FieldData, FieldMetadata, TryFromChunks, TryFromMultipart, TypedMultipart, TypedMultipartError,
};

use db::Question;

use sqlx::SqlitePool;

use crate::AppState;
use futures_util::stream::Stream;
use tempfile::NamedTempFile;

#[derive(TryFromMultipart)]
struct NewQuestion {
    category: Option<i64>,
    question: String,
    answer: String,
    #[form_data(limit = "1GiB")]
    attachment: FieldData<NamedTempFile>,
    hidden: FormBool,
}

struct FormBool(bool);

#[async_trait]
impl TryFromChunks for FormBool {
    async fn try_from_chunks(
        chunks: impl Stream<Item = Result<Bytes, TypedMultipartError>> + Send + Sync + Unpin,
        metadata: FieldMetadata,
    ) -> Result<Self, TypedMultipartError> {
        let string = String::try_from_chunks(chunks, metadata).await?;
        match string.as_str() {
            "on" => Ok(FormBool(true)),
            _ => Ok(FormBool(false)),
        }
    }
}

#[derive(Template)]
#[template(path = "questions/question_row.html", escape = "none")]
struct QuestionRow {
    question: Question,
}

#[derive(Template)]
#[template(path = "questions/question_row_edit.html", escape = "none")]
struct QuestionRowEdit {
    question: Question,
}

#[derive(Template)]
#[template(path = "questions/questions.html", escape = "none")]
struct QuestionsPage {
    questions: Vec<QuestionRow>,
}

#[derive(Template)]
#[template(path = "questions/questions_reordering.html", escape = "none")]
struct QuestionsReorderingPage {
    questions: Vec<Question>,
}

async fn get_questions(State(pool): State<SqlitePool>) -> impl IntoResponse {
    let questions = db::questions::get_all_questions(&pool).await.unwrap();
    QuestionsPage {
        questions: questions
            .into_iter()
            .map(|c| QuestionRow { question: c })
            .collect(),
    }
}

async fn create_question(
    State(pool): State<SqlitePool>,
    State(static_dir): State<PathBuf>,
    TypedMultipart(form): TypedMultipart<NewQuestion>,
) -> impl IntoResponse {
    let file_name = form
        .attachment
        .metadata
        .file_name
        .unwrap_or("random_name".to_owned());
    let id = db::questions::create_question(
        &pool,
        &form.question,
        &form.answer,
        form.category,
        vec![&file_name],
        form.hidden.0,
        0,
    )
    .await
    .unwrap();
    let path = static_dir.join(file_name);
    form.attachment.contents.persist(path).unwrap();
    QuestionRow {
        question: db::questions::get_question_by_id(&pool, id).await.unwrap(),
    }
}

// Small hack to cause redirect on client side so download modal will be displayed
// https://www.reddit.com/r/htmx/comments/pt4xng/htmxway_to_upload_and_download_a_file/
async fn download_attachment(Path(file_name): Path<String>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        format!("/static/{}", file_name).parse().unwrap(),
    );
    headers
}

pub fn questions_router(state: AppState) -> Router {
    Router::new()
        .route("/questions", get(get_questions).post(create_question))
        .route("/questions/download/:file_name", get(download_attachment))
        .with_state(state)
}
