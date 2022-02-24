mod private_chat;

use std::error::Error;
use std::sync::Arc;

use teloxide::types::PublicChatKind;
use teloxide::{
    dispatching2::dialogue::{serializer::Json, SqliteStorage},
    prelude2::*,
    types::ChatKind,
};

use tokio::sync::Mutex;

async fn handle_group_chat(_bot: AutoSend<Bot>) -> anyhow::Result<()> {
    log::info!("GroupMessage");
    Ok(())
}

fn filter_group_chats(upd: Update) -> bool {
    upd.chat()
        .and_then(|chat| match &chat.kind {
            ChatKind::Public(public_chat) => Some(public_chat),
            _ => None,
        })
        .and_then(|public_chat| match &public_chat.kind {
            PublicChatKind::Group(_) => Some(()),
            _ => None,
        })
        .is_some()
}

fn filter_private_chats(upd: Update) -> bool {
    upd.chat()
        .and_then(|chat| match &chat.kind {
            ChatKind::Private(_) => Some(()),
            _ => None,
        })
        .is_some()
}

async fn run() -> Result<(), Box<dyn Error>> {
    let conn = Arc::new(Mutex::new(db::establish_connection().await?));
    let bot = Bot::from_env().auto_send();
    let storage = SqliteStorage::open("db.sqlite", Json).await.unwrap();

    let handler = dptree::entry()
        .branch(dptree::filter(filter_group_chats).endpoint(handle_group_chat))
        .branch(
            dptree::filter(filter_private_chats).branch(private_chat::make_private_chat_branch()),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![conn, storage])
        .default_handler(|upd| async move {
            dbg!(upd);
        })
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    teloxide::enable_logging!();
    log::info!("Running db migrations...");
    db::run_migrations().await?;
    log::info!("Starting bot...");
    run().await?;
    log::info!("Closing bot... Goodbye!");
    Ok(())
}
