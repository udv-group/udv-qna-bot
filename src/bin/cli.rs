use clap::{Parser, Subcommand};
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::SqlitePool;
use std::error::Error;
use std::path::PathBuf;
use udv_qna_bot::db::queries::categories::{get_all_categories, import_categories};
use udv_qna_bot::db::queries::questions::{get_all_questions, import_questions};
use udv_qna_bot::db::queries::users::{get_users, import_users};
use udv_qna_bot::db::{Category, Question, User};
use udv_qna_bot::telemetry::init_tracing;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Database path
    db_path: PathBuf,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Import data to bot
    Import { path: PathBuf },
    /// Export data from bot
    Export { path: PathBuf },
}

#[tokio::main]
async fn main() {
    init_tracing();
    let cli = Cli::parse();
    let db_path: PathBuf = cli.db_path;
    let pool = SqlitePool::connect(format!("sqlite:{}", db_path.display()).as_str())
        .await
        .expect("Cannot connect to DB");
    match cli.command {
        Commands::Export { path } => export_data(&pool, path).await.expect("Cannot export"),
        Commands::Import { path } => import_data(&pool, path).await.expect("Cannot import"),
    }
}

fn write_to(path: PathBuf, data: Vec<impl Serialize>) -> Result<(), Box<dyn Error>> {
    let file = std::fs::File::create(path)?;
    let mut wtr = csv::Writer::from_writer(file);
    for line in data {
        wtr.serialize(line)?;
    }
    wtr.flush()?;
    Ok(())
}
fn read_from<T: DeserializeOwned>(path: PathBuf) -> Result<Vec<T>, Box<dyn Error>> {
    let file = std::fs::File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut out = Vec::new();
    for record in rdr.deserialize() {
        let record: T = record?;
        out.push(record);
    }
    Ok(out)
}
async fn export_data(pool: &SqlitePool, path: PathBuf) -> Result<(), Box<dyn Error>> {
    let categories = get_all_categories(pool).await?;
    let questions = get_all_questions(pool).await?;
    let users = get_users(pool).await?;
    if !path.exists() {
        std::fs::create_dir_all(&path)?
    }
    write_to(path.join("categories.csv"), categories)?;
    write_to(path.join("question.csv"), questions)?;
    write_to(path.join("users.csv"), users)?;
    Ok(())
}

async fn import_data(pool: &SqlitePool, path: PathBuf) -> Result<(), Box<dyn Error>> {
    let categories: Vec<Category> = read_from(path.clone().join("categories.csv"))?;
    let questions: Vec<Question> = read_from(path.clone().join("question.csv"))?;
    let users: Vec<User> = read_from(path.clone().join("users.csv"))?;
    import_users(pool, users).await?;
    import_categories(pool, categories).await?;
    import_questions(pool, questions).await?;
    Ok(())
}
