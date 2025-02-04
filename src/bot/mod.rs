mod auth;
mod private_chat;

use std::path::PathBuf;
use std::sync::Arc;

use sqlx::SqlitePool;
use teloxide::dispatching::dialogue::SqliteStorage;
use teloxide::{dispatching::dialogue::serializer::Json, prelude::*};

pub async fn run(pool: SqlitePool, chat_db_path: &str, static_dir: PathBuf) -> anyhow::Result<()> {
    let conn = Arc::new(pool);
    let storage = SqliteStorage::open(chat_db_path, Json).await.unwrap();

    let handler = dptree::entry().branch(
        dptree::filter(private_chat::filter_private_chats)
            .branch(private_chat::make_private_chat_branch()),
    );

    let mut builder = Dispatcher::builder(Bot::from_env(), handler)
        .dependencies(dptree::deps![conn, storage, Arc::new(static_dir)])
        .default_handler(|upd| async move {
            dbg!(upd);
        })
        .build();

    tracing::info!("Starting telegram bot");
    builder.dispatch().await;

    Ok(())
}
