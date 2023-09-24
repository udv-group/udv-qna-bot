use crate::deserializers::deserialize_bool_from_checkbox;
use anyhow::anyhow;
use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    async_trait,
    body::Bytes,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::get,
    Json, Router,
};
use axum_typed_multipart::{
    FieldData, FieldMetadata, TryFromChunks, TryFromMultipart, TypedMultipart, TypedMultipartError,
};
use serde_aux::prelude::deserialize_option_number_from_string;

use std::path::PathBuf;

use db::{Category, Question};

use serde::Deserialize;
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
    attachments: Vec<FieldData<NamedTempFile>>,
    hidden: Option<FormBool>,
}

#[derive(Deserialize)]
struct QuestionUpdate {
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_option_number_from_string")]
    category: Option<i64>,
    question: String,
    answer: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_bool_from_checkbox")]
    hidden: Option<bool>,
}

#[derive(Deserialize)]
struct QuestionsQuery {
    category: Option<i64>,
}

#[derive(TryFromMultipart)]
struct NewAttachments {
    #[form_data(limit = "1GiB")]
    attachment: FieldData<NamedTempFile>,
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
            unexpected => Err(TypedMultipartError::Other {
                source: anyhow!("Unexpected checkbox value {unexpected}"),
            }),
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
    categories: Vec<Category>,
    question: Question,
}

#[derive(Template)]
#[template(path = "questions/questions_table.html", escape = "none")]
struct QuestionsTable {
    questions: Vec<QuestionRow>,
}

#[derive(Template)]
#[template(path = "questions/questions.html", escape = "none")]
struct QuestionsPage {
    categories: Vec<Category>,
    table: QuestionsTable,
}

#[derive(Template)]
#[template(path = "questions/questions_reordering.html", escape = "none")]
struct QuestionsReorderingPage {
    questions: Vec<Question>,
}

#[derive(Template)]
#[template(path = "questions/attachments_modal.html", escape = "none")]
struct Attachments {
    id: i64,
    attachments: Vec<AttachmentRow>,
}

#[derive(Template)]
#[template(path = "questions/attachment_row.html", escape = "none")]
struct AttachmentRow {
    question_id: i64,
    name: String,
}

async fn questions_page(State(pool): State<SqlitePool>) -> impl IntoResponse {
    let questions = db::questions::get_all_questions(&pool).await.unwrap();
    let table = QuestionsTable {
        questions: questions
            .into_iter()
            .map(|c| QuestionRow { question: c })
            .collect(),
    };
    QuestionsPage {
        categories: db::categories::get_all_categories(&pool).await.unwrap(),
        table,
    }
}

async fn get_question(State(pool): State<SqlitePool>, Path(id): Path<i64>) -> impl IntoResponse {
    QuestionRow {
        question: db::questions::get_question_by_id(&pool, id).await.unwrap(),
    }
}

async fn questions_table(
    State(pool): State<SqlitePool>,
    Query(QuestionsQuery { category }): Query<QuestionsQuery>,
) -> impl IntoResponse {
    let questions = match category {
        Some(id) => db::questions::get_questions_for_category(&pool, id)
            .await
            .unwrap(),
        None => db::questions::get_all_questions(&pool).await.unwrap(),
    };
    QuestionsTable {
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
    let info: Vec<(String, NamedTempFile)> = form
        .attachments
        .into_iter()
        .map(|a| {
            let file_name = a.metadata.file_name.unwrap_or("random_name".to_owned());
            (file_name, a.contents)
        })
        .collect();

    let id = db::questions::create_question(
        &pool,
        &form.question,
        &form.answer,
        form.category,
        info.iter().map(|(a, _)| a.as_str()).collect(),
        form.hidden.map(|v| v.0).unwrap_or(false),
        0,
    )
    .await
    .unwrap();
    info.into_iter().for_each(|(name, contents)| {
        let question_dir = static_dir.join(id.to_string());
        std::fs::create_dir_all(&question_dir).unwrap();

        std::fs::copy(contents.path(), question_dir.join(name)).unwrap();
        std::fs::remove_file(contents.path()).unwrap();
    });

    QuestionRow {
        question: db::questions::get_question_by_id(&pool, id).await.unwrap(),
    }
}

async fn edit_question(State(pool): State<SqlitePool>, Path(id): Path<i64>) -> impl IntoResponse {
    let categories = db::categories::get_all_categories(&pool).await.unwrap();
    let question = db::questions::get_question_by_id(&pool, id).await.unwrap();
    QuestionRowEdit {
        categories,
        question,
    }
}

async fn update_question(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(form): Json<QuestionUpdate>,
) -> impl IntoResponse {
    let question = db::questions::get_question_by_id(&pool, id).await.unwrap();
    db::questions::update_question(
        &pool,
        id,
        form.category,
        form.question,
        form.answer,
        question.attachments.iter().map(|a| a.as_str()).collect(),
        form.hidden.unwrap_or(false),
    )
    .await
    .unwrap();
    let question = db::questions::get_question_by_id(&pool, id).await.unwrap();
    QuestionRow { question }
}

async fn delete_question(
    State(pool): State<SqlitePool>,
    State(static_dir): State<PathBuf>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    db::questions::delete_question(&pool, id).await.unwrap();
    std::fs::remove_dir_all(static_dir.join(id.to_string())).unwrap();
}

// Small hack to cause redirect on client side so download modal will be displayed
async fn download_attachment(Path((id, file_name)): Path<(i64, String)>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        format!("/static/{}/{}", id, file_name).parse().unwrap(),
    );
    headers
}

async fn attachments(State(pool): State<SqlitePool>, Path(id): Path<i64>) -> impl IntoResponse {
    let question = db::questions::get_question_by_id(&pool, id).await.unwrap();
    Attachments {
        id,
        attachments: question
            .attachments
            .into_iter()
            .map(|a| AttachmentRow {
                question_id: id,
                name: a,
            })
            .collect(),
    }
}

async fn delete_attachment(
    State(static_dir): State<PathBuf>,
    State(pool): State<SqlitePool>,
    Path((id, file_name)): Path<(i64, String)>,
) -> impl IntoResponse {
    let mut question = db::questions::get_question_by_id(&pool, id).await.unwrap();
    question.attachments.sort();
    let idx = question.attachments.binary_search(&file_name).unwrap();
    question.attachments.remove(idx);
    db::questions::update_question(
        &pool,
        id,
        question.category,
        question.question,
        question.answer,
        question.attachments.iter().map(|a| a.as_str()).collect(),
        question.hidden,
    )
    .await
    .unwrap();

    std::fs::remove_file(static_dir.join(id.to_string()).join(file_name)).unwrap();
}

async fn add_attachment(
    State(static_dir): State<PathBuf>,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    TypedMultipart(form): TypedMultipart<NewAttachments>,
) -> impl IntoResponse {
    let file_name = form
        .attachment
        .metadata
        .file_name
        .unwrap_or("random".to_owned());
    let mut question = db::questions::get_question_by_id(&pool, id).await.unwrap();
    question.attachments.push(file_name.clone());
    db::questions::update_question(
        &pool,
        id,
        question.category,
        question.question,
        question.answer,
        question.attachments.iter().map(|a| a.as_str()).collect(),
        question.hidden,
    )
    .await
    .unwrap();
    let question_dir = static_dir.join(id.to_string());
    std::fs::create_dir_all(&question_dir).unwrap();

    std::fs::copy(
        form.attachment.contents.path(),
        question_dir.join(&file_name),
    )
    .unwrap();
    std::fs::remove_file(form.attachment.contents.path()).unwrap();
    AttachmentRow {
        question_id: id,
        name: file_name,
    }
}

pub fn questions_router(state: AppState) -> Router {
    Router::new()
        .route("/questions", get(questions_page).post(create_question))
        .route("/questions/table", get(questions_table))
        .route("/questions/:id/edit", get(edit_question))
        .route(
            "/questions/:id",
            get(get_question)
                .delete(delete_question)
                .put(update_question),
        )
        .route(
            "/questions/:id/attachments",
            get(attachments).post(add_attachment),
        )
        .route(
            "/questions/:id/attachments/:file_name",
            get(download_attachment).delete(delete_attachment),
        )
        .with_state(state)
}
