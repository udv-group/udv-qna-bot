use anyhow::bail;
use db::Question;
use itertools::Itertools;
use sqlx::SqlitePool;
use std::borrow::Borrow;

use std::path::PathBuf;
use std::sync::Arc;

use teloxide::{
    dispatching::dialogue::{serializer::Json, SqliteStorage},
    dispatching::DpHandlerDescription,
    prelude::*,
    types::{InputFile, KeyboardButton, KeyboardMarkup, KeyboardRemove},
    utils::command::BotCommands,
};

use serde::{Deserialize, Serialize};
use teloxide::types::{ChatKind, ChatMemberKind};

use crate::auth;

type MyDialogue = Dialogue<State, SqliteStorage<Json>>;

#[derive(BotCommands, Clone)]
#[command(description = "These commands are supported:")]
enum Command {
    #[command(rename = "lowercase", description = "Display this text")]
    Help,
    #[command(rename = "lowercase", description = "Start")]
    Start,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum State {
    // In this state list of Categories is displayed on the keyboard
    ShowingCategories,
    // In this state list of Questions is displayed on the keyboard
    ShowingQuestions { category: String },
}

impl Default for State {
    fn default() -> Self {
        Self::ShowingCategories
    }
}

async fn make_categories_keyboard(conn: &SqlitePool) -> anyhow::Result<KeyboardMarkup> {
    let results: Vec<String> = db::categories::get_public_categories(conn)
        .await?
        .into_iter()
        .map(|category_| category_.name)
        .collect();
    make_keyboard(results, 2, false)
}

async fn make_questions_keyboard(
    conn: &SqlitePool,
    category: &str,
) -> anyhow::Result<KeyboardMarkup> {
    let results: Vec<String> =
        db::questions::get_public_questions_for_public_category(conn, category)
            .await?
            .into_iter()
            .map(|question| question.question)
            .collect();
    if results.is_empty() {
        bail!(
            "Category {} is unknown or has no available questions",
            category
        );
    }
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

async fn reply_with_answer(
    bot: AutoSend<Bot>,
    msg: Message,
    static_dir: Arc<PathBuf>,
    question: Question,
) -> anyhow::Result<()> {
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
    for att in question.attachments.iter() {
        let filepath = static_dir.join(att);
        if filepath.is_file() {
            bot.send_document(msg.chat.id, InputFile::file(filepath))
                .await?;
        } else {
            log::error!("File {:#?} is not found!", filepath);
        }
    };
    Ok(())
}

async fn on_question_select(
    bot: AutoSend<Bot>,
    msg: Message,
    dialogue: MyDialogue,
    category: String,
    conn: Arc<SqlitePool>,
    static_dir: Arc<PathBuf>,
) -> anyhow::Result<()> {
    let text = match msg.text() {
        Some(text) => text,
        None => {
            bot.send_message(msg.chat.id, "Please select the question".to_string())
                .reply_markup(make_questions_keyboard(conn.borrow(), category.as_str()).await?)
                .await?;
            return Ok(());
        }
    };
    match text {
        "Go Back" => {
            dialogue.reset().await?;
            bot.send_message(msg.chat.id, "Main menu")
                .reply_markup(make_categories_keyboard(conn.borrow()).await?)
                .await?;
        }
        selected_question => {
            match db::questions::get_question(conn.borrow(), selected_question, &category).await {
                Ok(question) => reply_with_answer(bot, msg, static_dir, question).await?,
                Err(_) => {
                    bot.send_message(
                        msg.chat.id,
                        format!("Unknown question: {}", selected_question),
                    )
                    .reply_markup(make_questions_keyboard(conn.borrow(), category.as_str()).await?)
                    .await?;
                }
            };
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
    let category = match msg.text() {
        Some(text) => text,
        None => {
            bot.send_message(msg.chat.id, "Please select the category")
                .reply_markup(make_categories_keyboard(conn.borrow()).await?)
                .await?;
            return Ok(());
        }
    };
    match make_questions_keyboard(conn.borrow(), category).await {
        Ok(keyboard) => {
            bot.send_message(msg.chat.id, format!("You chose category {}", category))
                .reply_markup(keyboard)
                .await?;
        }
        Err(e) => {
            log::warn!("Exception getting category {}", e);
            bot.send_message(
                msg.chat.id,
                format!("Category {} is unknown or has no questions", category),
            )
            .await?;
            return Ok(());
        }
    }
    dialogue
        .update(State::ShowingQuestions {
            category: category.to_string(),
        })
        .await?;
    Ok(())
}

async fn handle_private_chat_member(
    _bot: AutoSend<Bot>,
    msg: ChatMemberUpdated,
    storage: Arc<SqliteStorage<Json>>,
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
        }
        kind => log::info!("Unsupported member kind{:?}", kind),
    }
    Ok(())
}

async fn on_commands(
    bot: AutoSend<Bot>,
    msg: Message,
    cmd: Command,
    dialogue: MyDialogue,
    conn: Arc<SqlitePool>,
) -> anyhow::Result<()> {
    match cmd {
        Command::Start => {
            dialogue.reset().await?;
            bot.send_message(msg.chat.id, "Main menu")
                .reply_markup(make_categories_keyboard(conn.borrow()).await?)
                .await?
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
    };

    Ok(())
}

async fn handle_not_authenticated(bot: AutoSend<Bot>, msg: Message) -> anyhow::Result<()> {
    bot.send_message(
        msg.chat.id,
        "You are not authorized to use this bot, contact the admin for authentication",
    )
    .reply_markup(KeyboardRemove::new())
    .await?;
    Ok(())
}

// return true when user is _not_ authenticated
async fn auth_failed(msg: Message, conn: Arc<SqlitePool>) -> bool {
    !auth::auth_user(&conn, msg.from().expect("Got message not from a user?"))
        .await
        .map_err(|err| {
            log::warn!("Unable to authenticate user {:?}: {}", msg.from(), err);
            err
        })
        .unwrap_or(false)
}

pub fn make_private_chat_branch(
) -> Handler<'static, DependencyMap, anyhow::Result<()>, DpHandlerDescription> {
    let commands_handler = dptree::entry()
        .filter_command::<Command>()
        .endpoint(on_commands);

    let dialogues_handler = dptree::entry()
        .branch(dptree::case![State::ShowingCategories].endpoint(on_category_select))
        .branch(dptree::case![State::ShowingQuestions { category }].endpoint(on_question_select));

    // if user is not authenticated - display "blocked" message
    let auth_handler = dptree::entry()
        .filter_async(
            |msg: Message, conn: Arc<SqlitePool>| async move { auth_failed(msg, conn).await },
        )
        .endpoint(handle_not_authenticated);

    let messages_handler = dptree::entry()
        .enter_dialogue::<Message, SqliteStorage<Json>, State>()
        .branch(auth_handler)
        .branch(commands_handler)
        .branch(dialogues_handler);

    return dptree::entry()
        .branch(Update::filter_message().chain(messages_handler))
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
