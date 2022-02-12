use diesel::sqlite::SqliteConnection;
use std::borrow::Borrow;

use std::error::Error;
use std::sync::Arc;

use teloxide::{
    payloads::SendMessageSetters,
    prelude2::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommand,
};

use serde::{Deserialize, Serialize};

use serde_repr::{Deserialize_repr, Serialize_repr};
use tokio::sync::Mutex;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Start")]
    Start,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Clone, Copy)]
#[repr(u8)]
enum KeyboardType {
    Question = 1,
    Category = 2,
    Answer = 3,
}

#[derive(Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum StateSwitch {
    Next = 1,
    Back = 2,
}

#[derive(Serialize, Deserialize)]
struct CallbackInfo {
    // id of the element being displayed
    #[serde(rename = "i")]
    button_id: i32,
    // current state of the keyboard on user's screen
    #[serde(rename = "kt")]
    keyboard_type: KeyboardType,
    // where to move from here
    #[serde(rename = "s")]
    state: StateSwitch,
    // ???
    #[serde(rename = "psi")]
    previous_state_id: Option<i32>,
}

//todo: this api is garbage, fix it
/// Creates a keyboard made by buttons in a big column.
fn make_keyboard(
    button_info: Vec<(String, i32)>,
    current_keyboard_id: Option<i32>,
    keyboard_type: KeyboardType,
) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    for text in button_info.chunks(3) {
        let row = text
            .iter()
            .map(|(text, id)| {
                let callback_info = CallbackInfo {
                    button_id: *id,
                    keyboard_type,
                    state: StateSwitch::Next,
                    previous_state_id: current_keyboard_id,
                };
                InlineKeyboardButton::callback(
                    text.to_owned(),
                    serde_json::to_string(&callback_info).unwrap(),
                )
            })
            .collect();

        keyboard.push(row);
    }
    if (keyboard_type == KeyboardType::Answer) | (keyboard_type == KeyboardType::Question) {
        let callback_info = CallbackInfo {
            button_id: 42,
            keyboard_type,
            state: StateSwitch::Back,
            previous_state_id: current_keyboard_id,
        };
        keyboard.push(vec![InlineKeyboardButton::callback(
            "Go Back".to_string(),
            serde_json::to_string(&callback_info).unwrap(),
        )]);
    }

    InlineKeyboardMarkup::new(keyboard)
}

/// Parse the text wrote on Telegram and check if that text is a valid command
/// or not, then match the command. If the command is `/start` it writes a
/// markup with the `InlineKeyboardMarkup`.
async fn message_handler(
    msg: Message,
    bot: AutoSend<Bot>,
    conn: Arc<Mutex<SqliteConnection>>,
) -> Result<(), teloxide::RequestError> {
    if let Some(text) = msg.text() {
        match BotCommand::parse(text, "buttons") {
            Ok(Command::Help) => {
                // Just send the description of all commands.
                bot.send_message(msg.chat.id, Command::descriptions())
                    .await?;
            }
            Ok(Command::Start) => {
                // Create a list of buttons and send them.
                let results: Vec<(String, i32)> = db::get_categories(conn.lock().await.borrow())
                    .into_iter()
                    .map(|category_| (category_.name, category_.id))
                    .collect();
                let keyboard = make_keyboard(results, None, KeyboardType::Category);
                bot.send_message(msg.chat.id, "Available categories:")
                    .reply_markup(keyboard)
                    .await?;
            }

            Err(_) => {
                log::info!("{}", text);
                bot.send_message(msg.chat.id, "Command not found!").await?;
            }
        }
    }

    Ok(())
}

async fn update_message<T: Into<String>>(
    bot: AutoSend<Bot>,
    message: Option<Message>,
    text: T,
    keyboard: InlineKeyboardMarkup,
) -> Result<(), teloxide::RequestError> {
    match message {
        Some(Message { id, chat, .. }) => {
            bot.edit_message_text(chat.id, id, text)
                .reply_markup(keyboard)
                .await?;
        }
        None => {
            log::warn!("No message, wtf");
        }
    }
    Ok(())
}

/// When it receives a callback from a button it edits the message with all
/// those buttons writing a text with the selected Debian version.
async fn callback_handler(
    query: CallbackQuery,
    bot: AutoSend<Bot>,
    conn: Arc<Mutex<SqliteConnection>>,
) -> Result<(), teloxide::RequestError> {
    if let Some(data) = query.data {
        log::info!("Selected: {}", data);
        let callback_info: CallbackInfo = serde_json::from_str(&data).unwrap();
        match callback_info.keyboard_type {
            // if user tapped category button switch to questions in that category
            // user can only go Next
            KeyboardType::Category => {
                let results: Vec<(String, i32)> = db::get_questions_by_category(
                    conn.lock().await.borrow(),
                    callback_info.button_id,
                ) // todo: filter by category
                .into_iter()
                .map(|item| (item.question, item.id))
                .collect();
                let keyboard = make_keyboard(
                    results,
                    Some(callback_info.button_id),
                    KeyboardType::Question,
                );
                update_message(bot, query.message, "Select a question", keyboard).await?;
            }
            // if user tapped "Go Back" on the answer - return to the questions list
            // user can only go Back
            KeyboardType::Answer => {
                let results: Vec<(String, i32)> = db::get_questions_by_category(
                    conn.lock().await.borrow(),
                    callback_info.previous_state_id.unwrap(),
                ) // todo: filter by category
                .into_iter()
                .map(|item| (item.question, item.id))
                .collect();
                let keyboard = make_keyboard(results, None, KeyboardType::Question);
                update_message(bot, query.message, "Select a question", keyboard).await?;
            }
            // if user tapped something on questions keyboard
            KeyboardType::Question => match callback_info.state {
                // user selected a question, show the answer, set the Go Back button id as this question id
                StateSwitch::Next => {
                    let question =
                        db::get_question_by_id(conn.lock().await.borrow(), callback_info.button_id);
                    let keyboard = make_keyboard(vec![], question.category, KeyboardType::Answer);
                    update_message(bot, query.message, question.answer, keyboard).await?;
                }
                // user wants to go back to category selection
                StateSwitch::Back => {
                    let results: Vec<(String, i32)> =
                        db::get_categories(conn.lock().await.borrow())
                            .into_iter()
                            .map(|category_| (category_.name, category_.id))
                            .collect();
                    let keyboard = make_keyboard(results, None, KeyboardType::Category);
                    update_message(bot, query.message, "Available categories", keyboard).await?;
                }
            },
        }
    };

    Ok(())
}

async fn run() -> Result<(), Box<dyn Error>> {
    let bot = Bot::from_env().auto_send();
    // TODO: share the connection somehow
    let conn = Arc::new(Mutex::new(db::establish_connection()?));
    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![conn])
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    teloxide::enable_logging!();
    log::info!("Starting bot...");
    run().await?;
    log::info!("Closing bot... Goodbye!");
    Ok(())
}
