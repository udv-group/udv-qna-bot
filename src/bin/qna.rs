use std::{fs::create_dir_all, path::PathBuf};

use anyhow::Context;
use udv_qna_bot::db::run_migrations;
use udv_qna_bot::server::app::run_server;
use udv_qna_bot::telemetry::init_tracing;
use udv_qna_bot::{bot::run, db};

use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(default_value = "all")]
    runner: Runner,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum Runner {
    Server,
    Bot,
    All,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let cli = Cli::parse();
    let path = dotenv::var("DB_PATH").expect("DB_PATH must be set");
    let pool = db::establish_connection(&path).await.unwrap();
    let static_dir =
        PathBuf::from(dotenv::var("STATIC_DIR").expect("Variable STATIC_DIR should be set"));
    if !static_dir.exists() {
        create_dir_all(&static_dir).context("Failed to create directory for static content")?;
    }
    if !static_dir.is_dir() {
        anyhow::bail!("Variable STATIC_DIR should be a directory or not exist");
    }

    tracing::info!("Running db migrations...");
    run_migrations().await?;

    match cli.runner {
        Runner::Server => run_server(pool.clone(), static_dir.clone()).await?,
        Runner::Bot => run(pool.clone(), &path, static_dir).await?,
        Runner::All => {
            tokio::select! {
               res = run(pool.clone(), &path, static_dir.clone()) => {tracing::warn!("Bot exited: {:#?}", res)}
               res = run_server(pool.clone(), static_dir.clone()) => {tracing::warn!("Server exited: {:#?}", res)}
            }
        }
    };
    Ok(())
}
