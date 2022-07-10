use itertools::Itertools;
use sqlx::SqlitePool;
use std::borrow::Borrow;

use std::path::PathBuf;
use std::sync::Arc;

use teloxide::{
    dispatching::dialogue::{serializer::Json, SqliteStorage},
    dispatching::DpHandlerDescription,
    prelude::*,
    types::{InputFile, KeyboardButton, KeyboardMarkup},
    utils::command::BotCommands,
};

use serde::{Deserialize, Serialize};
use teloxide::types::{ChatKind, ChatMemberKind};

use crate::auth;

type MyDialogue = Dialogue<State, SqliteStorage<Json>>;

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Start")]
    Start,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum State {
    ShowingCategories,
    ShowingQuestions { category: String },
    Blocked,
}

impl Default for State {
    fn default() -> Self {
        Self::ShowingCategories
    }
}

async fn make_main_menu(conn: &SqlitePool) -> anyhow::Result<KeyboardMarkup> {
    let results: Vec<String> = db::categories::get_categories(conn)
        .await?
        .into_iter()
        .map(|category_| category_.name)
        .collect();
    make_keyboard(results, 2, false)
}

async fn make_questions(conn: &SqlitePool, category: &str) -> anyhow::Result<KeyboardMarkup> {
    let results: Vec<String> = db::questions::get_questions_by_category(conn, category)
        .await?
        .into_iter()
        .map(|question| question.question)
        .collect();

    make_keyboard(results, 1, true)
}

fn make_keyboard(
    data: Vec<String>,
    rows: usize,
    with_exit: bool,
) -> anyhow::Result<KeyboardMarkup> {
    let mut keyboard: Vec<Vec<KeyboardButton>> = vec![];

    for text in data.chunks(rows) {
        let row = text.iter().map(KeyboardButton::new).collect();

        keyboard.push(row);
    }
    if with_exit {
        keyboard.push(vec![KeyboardButton::new("Go Back")]);
    }
    Ok(KeyboardMarkup::new(keyboard))
}

async fn on_question_select(
    bot: AutoSend<Bot>,
    msg: Message,
    dialogue: MyDialogue,
    category: String,
    conn: Arc<SqlitePool>,
    static_dir: Arc<PathBuf>,
) -> anyhow::Result<()> {
    let text = msg.text().unwrap_or("unknown");
    match text {
        "Go Back" => {
            dialogue.reset().await?;
            bot.send_message(msg.chat.id, "Main menu")
                .reply_markup(make_main_menu(conn.borrow()).await?)
                .await?;
        }
        selected_question => {
            if let Ok(question) =
                db::questions::get_question(conn.borrow(), selected_question).await
            {
                let data_v: Vec<String> = question
                    .answer
                    .chars()
                    .chunks(2048)
                    .into_iter()
                    .map(|chunk| chunk.collect::<String>())
                    .collect();
                for data in data_v {
                    bot.send_message(msg.chat.id, data).await.unwrap();
                }
                if let Some(att) = question.attachment {
                    let filepath = static_dir.join(att);
                    if filepath.is_file() {
                        bot.send_document(msg.chat.id, InputFile::file(filepath))
                            .await?;
                    } else {
                        log::error!("File {:#?} is not found!", filepath);
                    }
                }
            } else {
                bot.send_message(msg.chat.id, "Question does not exist".to_string())
                    .reply_markup(make_questions(conn.borrow(), category.as_str()).await?)
                    .await?;
            }
        }
    }
    Ok(())
}

async fn on_category_select(
    bot: AutoSend<Bot>,
    msg: Message,
    dialogue: MyDialogue,
    conn: Arc<SqlitePool>,
) -> anyhow::Result<()> {
    let category = msg.text().unwrap_or("unknown");
    dialogue
        .update(State::ShowingQuestions {
            category: category.to_string(),
        })
        .await?;
    match make_questions(conn.borrow(), category).await {
        Ok(keyboard) => {
            bot.send_message(msg.chat.id, format!("You chose category {}", category))
                .reply_markup(keyboard)
                .await?;
        }
        Err(e) => {
            log::warn!("Exception getting category {}", e);
            bot.send_message(msg.chat.id, format!("Unknown category: {}", category))
                .await?;
        }
    }
    Ok(())
}

async fn handle_private_chat_member(
    _bot: AutoSend<Bot>,
    msg: ChatMemberUpdated,
    storage: Arc<SqliteStorage<Json>>,
    conn: Arc<SqlitePool>,
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
            if !auth::auth_user(conn.borrow(), msg.from.id.0).await {
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
    conn: Arc<SqlitePool>,
) -> anyhow::Result<()> {
    match cmd {
        Command::Start => {
            if !auth::auth_user(conn.borrow(), msg.from().unwrap().id.0).await {
                dialogue.update(State::Blocked).await?;
                handle_blocked(bot, msg).await?;
                return Ok(());
            }
            dialogue.reset().await?;
            bot.send_message(msg.chat.id, "Main menu")
                .reply_markup(make_main_menu(conn.borrow()).await?)
                .await?
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
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

pub fn make_private_chat_branch(
) -> Handler<'static, DependencyMap, anyhow::Result<()>, DpHandlerDescription> {
    let commands_handler = dptree::entry()
        .enter_dialogue::<Message, SqliteStorage<Json>, State>()
        .filter_command::<Command>()
        .endpoint(handle_commands);

    let dialogues_handler = dptree::entry()
        .enter_dialogue::<Message, SqliteStorage<Json>, State>()
        .branch(dptree::case![State::ShowingCategories].endpoint(on_category_select))
        .branch(dptree::case![State::ShowingQuestions { category }].endpoint(on_question_select))
        .branch(dptree::case![State::Blocked].endpoint(handle_blocked));

    let messages_handler = Update::filter_message()
        .branch(commands_handler)
        .branch(dialogues_handler);

    return dptree::entry()
        .branch(messages_handler)
        .branch(Update::filter_my_chat_member().endpoint(handle_private_chat_member));
}

pub fn filter_private_chats(upd: Update) -> bool {
    upd.chat()
        .and_then(|chat| match &chat.kind {
            ChatKind::Private(_) => Some(()),
            _ => None,
        })
        .is_some()
}
