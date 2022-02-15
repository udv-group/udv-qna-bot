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
use teloxide::types::ChatMemberKind;

use tokio::sync::Mutex;

type MyDialogue = Dialogue<State, SqliteStorage<Json>>;
type StorageError = <SqliteStorage<Json> as Storage<State>>::Error;

#[derive(BotCommand, Clone)]
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
    #[handler(handle_blocked)]
    Blocked,
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
            let text = match db::get_question(conn.lock().await.borrow(), question).await {
                Ok(question) => question.answer,
                Err(_) => "Question does not exist".to_string(),
            };
            bot.send_message(msg.chat.id, text)
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
    dialogue
        .update(State::ShowQuestions(category.to_string()))
        .await?;
    //FIXME: inform user about unknown category, if typed and not selected on keyboard
    bot.send_message(msg.chat.id, format!("You chose category {}", category))
        .reply_markup(make_questions(conn.lock().await.borrow(), category).await?)
        .await?;
    Ok(())
}

async fn handle_my_chat_member(
    bot: AutoSend<Bot>,
    msg: ChatMemberUpdated,
    storage: Arc<SqliteStorage<Json>>,
    conn: Arc<Mutex<SqlitePool>>,
) -> anyhow::Result<()> {
    let dialogue: Dialogue<State, SqliteStorage<Json>> = Dialogue::new(storage, msg.chat.id);
    match msg.new_chat_member.kind {
        ChatMemberKind::Banned(_) => {
            log::info!(
                "User {:?} has blocked the bot, deleting dialogue state",
                msg.chat.username()
            );
            dialogue.exit().await?;
        }
        ChatMemberKind::Member => {
            log::info!("New user {:?} connected", msg.from);
            if db::get_user(conn.lock().await.borrow(), msg.from.id)
                .await
                .is_err()
            {
                dialogue.update(State::Blocked).await?;
            }
        }
        kind => log::info!("Unsupported member kind{:?}", kind),
    }
    Ok(())
}

async fn handle_commands(
    bot: AutoSend<Bot>,
    msg: Message,
    cmd: Command,
    dialogue: MyDialogue,
    conn: Arc<Mutex<SqlitePool>>,
) -> anyhow::Result<()> {
    match cmd {
        Command::Start => {
            if db::get_user(conn.lock().await.borrow(), msg.from().unwrap().id)
                .await
                .is_err()
            {
                dialogue.update(State::Blocked).await?;
                bot.send_message(
                    msg.chat.id,
                    "You are not authorized to use this bot, contact the admin for authentication",
                )
                .await?;
                return Ok(());
            }
            dialogue.reset().await?;
            bot.send_message(msg.chat.id, "Main menu")
                .reply_markup(make_main_menu(conn.lock().await.borrow()).await?)
                .await?
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions())
                .await?
        }
    };

    Ok(())
}

async fn handle_blocked(bot: AutoSend<Bot>, msg: Message) -> anyhow::Result<()> {
    bot.send_message(
        msg.chat.id,
        "You are not authorized to use this bot, contact the admin for authentication",
    )
    .await?;
    Ok(())
}

async fn run() -> Result<(), Box<dyn Error>> {
    let conn = Arc::new(Mutex::new(db::establish_connection().await?));
    let bot = Bot::from_env().auto_send();
    let storage = SqliteStorage::open("db.sqlite", Json).await.unwrap();

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .branch(
                    dptree::entry()
                        .enter_dialogue::<Message, SqliteStorage<Json>, State>()
                        .filter_command::<Command>()
                        .endpoint(handle_commands),
                )
                .branch(
                    dptree::entry()
                        .enter_dialogue::<Message, SqliteStorage<Json>, State>()
                        .dispatch_by::<State>(),
                ),
        )
        .branch(Update::filter_my_chat_member().endpoint(handle_my_chat_member));

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
