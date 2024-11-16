mod auth;
mod group_chat;
mod private_chat;

use std::sync::Arc;
use std::{fs::create_dir_all, path::PathBuf};

use teloxide::dispatching::dialogue::SqliteStorage;
use teloxide::{dispatching::dialogue::serializer::Json, prelude::*};

use crate::db::{establish_connection, run_migrations};

async fn run() -> anyhow::Result<()> {
    dotenv::var("USE_AUTH")
        .expect("Variable USE_AUTH should be set")
        .parse::<bool>()
        .expect("Should be 'true' or 'false'");
    let static_dir =
        PathBuf::from(dotenv::var("STATIC_DIR").expect("Variable STATIC_DIR should be set"));
    if !static_dir.is_dir() {
        panic!("Variable STATIC_DIT should be a directory or not exist");
    }
    if !static_dir.exists() {
        create_dir_all(&static_dir).unwrap();
    }
    let db_path = dotenv::var("DB_PATH").expect("DB_PATH must be set");
    let conn = Arc::new(establish_connection(&db_path).await?);
    let storage = SqliteStorage::open(&db_path, Json).await.unwrap();

    let handler = dptree::entry()
        .branch(
            dptree::filter(group_chat::filter_group_chats)
                .branch(group_chat::make_group_chat_branch()),
        )
        .branch(
            dptree::filter(private_chat::filter_private_chats)
                .branch(private_chat::make_private_chat_branch()),
        );

    Dispatcher::builder(Bot::from_env(), handler)
        .enable_ctrlc_handler()
        .dependencies(dptree::deps![conn, storage, Arc::new(static_dir)])
        .default_handler(|upd| async move {
            dbg!(upd);
        })
        .build()
        .dispatch()
        .await;

    Ok(())
}

pub async fn start_bot() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing::info!("Running db migrations...");
    run_migrations().await?;
    tracing::info!("Starting bot...");
    run().await?;
    tracing::info!("Closing bot... Goodbye!");
    Ok(())
}
