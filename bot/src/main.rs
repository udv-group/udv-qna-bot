use sqlx::SqlitePool;
use std::borrow::Borrow;

use std::error::Error;
use std::sync::Arc;

use teloxide::{
    dispatching2::dialogue::{serializer::Json, SqliteStorage, Storage},
    macros::DialogueState,
    payloads::SendMessageSetters,
    prelude2::*,
    types::{KeyboardButton, KeyboardMarkup},
    utils::command::BotCommand,
};

use serde::{Deserialize, Serialize};

use tokio::sync::Mutex;

type MyDialogue = Dialogue<State, SqliteStorage<Json>>;
type StorageError = <SqliteStorage<Json> as Storage<State>>::Error;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Start")]
    Start,
}

#[derive(DialogueState, Clone, Serialize, Deserialize)]
#[handler_out(anyhow::Result<()>)]
pub enum State {
    #[handler(handle_main_menu)]
    Start,
    #[handler(handle_show_questions)]
    ShowQuestions(String),
}

impl Default for State {
    fn default() -> Self {
        Self::Start
    }
}

async fn make_main_menu(conn: &SqlitePool) -> anyhow::Result<KeyboardMarkup> {
    let results: Vec<String> = db::get_categories(conn)
        .await?
        .into_iter()
        .map(|category_| category_.name)
        .collect();
    let mut keyboard: Vec<Vec<KeyboardButton>> = vec![];

    for text in results.chunks(2) {
        let row = text.iter().map(KeyboardButton::new).collect();

        keyboard.push(row);
    }
    Ok(KeyboardMarkup::new(keyboard))
}

async fn make_questions(conn: &SqlitePool, category: &str) -> anyhow::Result<KeyboardMarkup> {
    let results: Vec<String> = db::get_questions_by_category(conn, category)
        .await?
        .into_iter()
        .map(|question| question.question)
        .collect();

    let mut keyboard: Vec<Vec<KeyboardButton>> = results
        .iter()
        .map(|text| vec![KeyboardButton::new(text)])
        .collect();

    keyboard.push(vec![KeyboardButton::new("Go Back")]);
    Ok(KeyboardMarkup::new(keyboard))
}

async fn handle_show_questions(
    bot: AutoSend<Bot>,
    msg: Message,
    dialogue: MyDialogue,
    category: String,
    conn: Arc<Mutex<SqlitePool>>,
) -> anyhow::Result<()> {
    let text = msg.text().unwrap();
    match text {
        "Go Back" => {
            dialogue.reset().await?;
            bot.send_message(msg.chat.id, "Main menu")
                .reply_markup(make_main_menu(conn.lock().await.borrow()).await?)
                .await?;
        }
        question => {
            let question = db::get_question(conn.lock().await.borrow(), question).await?;
            bot.send_message(msg.chat.id, question.answer)
                .reply_markup(make_questions(conn.lock().await.borrow(), category.as_str()).await?)
                .await?;
        }
    }
    Ok(())
}

async fn handle_main_menu(
    bot: AutoSend<Bot>,
    msg: Message,
    dialogue: MyDialogue,
    conn: Arc<Mutex<SqlitePool>>,
) -> anyhow::Result<()> {
    let category = msg.text().unwrap();
    if !category.starts_with('/') {
        dialogue
            .update(State::ShowQuestions(category.to_string()))
            .await?;
        bot.send_message(msg.chat.id, format!("You chose category {}", category))
            .reply_markup(make_questions(conn.lock().await.borrow(), category).await?)
            .await?;
    } else {
        bot.send_message(msg.chat.id, "Main menu")
            .reply_markup(make_main_menu(conn.lock().await.borrow()).await?)
            .await?;
    }
    Ok(())
}

async fn run() -> Result<(), Box<dyn Error>> {
    let conn = Arc::new(Mutex::new(db::establish_connection().await?));
    let bot = Bot::from_env().auto_send();
    let storage = SqliteStorage::open("db.sqlite", Json).await.unwrap();

    let handler = Update::filter_message()
        .enter_dialogue::<Message, SqliteStorage<Json>, State>()
        .dispatch_by::<State>();

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![conn, storage])
        .default_handler(|upd| async move {
            log::warn!("Unhandled update: {:?}", upd);
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
