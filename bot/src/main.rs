use std::error::Error;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommand,
};

use serde::{Deserialize, Serialize};

use serde_repr::{Deserialize_repr, Serialize_repr};
use tokio_stream::wrappers::UnboundedReceiverStream;

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
    button_id: i32,
    // current state of the keyboard on user's screen
    keyboard_type: KeyboardType,
    // where to move from here
    state: StateSwitch,
}

/// Creates a keyboard made by buttons in a big column.
fn make_keyboard(button_text: Vec<String>, keyboard_type: KeyboardType) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    for text in button_text.chunks(3) {
        let row = text
            .iter()
            .map(|text| {
                let callback_info = CallbackInfo {
                    button_id: text.len() as i32, // placeholder, going to be the id of the DB element
                    keyboard_type,
                    state: StateSwitch::Next,
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
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = cx.update.text() {
        match BotCommand::parse(text, "buttons") {
            Ok(Command::Help) => {
                // Just send the description of all commands.
                cx.answer(Command::descriptions()).await?;
            }
            Ok(Command::Start) => {
                // Create a list of buttons and send them.
                let words = vec!["Foo1".to_owned(), "Bar2".to_owned()];
                let keyboard = make_keyboard(words, KeyboardType::Category);
                cx.answer("Available categories:")
                    .reply_markup(keyboard)
                    .await?;
            }

            Err(_) => {
                cx.reply_to("Command not found!").await?;
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
) -> Result<(), Box<dyn Error + Send + Sync>> {
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
    cx: UpdateWithCx<AutoSend<Bot>, CallbackQuery>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let UpdateWithCx {
        requester: bot,
        update: query,
    } = cx;
    if let Some(data) = query.data {
        log::info!("Selected: {}", data);
        let callback_info: CallbackInfo = serde_json::from_str(&data)?;
        match callback_info.keyboard_type {
            KeyboardType::Category | KeyboardType::Answer => {
                let keyboard = make_keyboard(
                    vec!["Question1".to_owned(), "Question2".to_owned()],
                    KeyboardType::Question,
                );
                update_message(bot, query.message, "Select a question", keyboard).await?;
            }
            KeyboardType::Question => match callback_info.state {
                StateSwitch::Next => {
                    let keyboard = make_keyboard(vec![], KeyboardType::Answer);
                    update_message(bot, query.message, "Some answer", keyboard).await?;
                }
                StateSwitch::Back => {
                    let keyboard = make_keyboard(
                        vec!["Foo1".to_owned(), "Bar2".to_owned()],
                        KeyboardType::Category,
                    );
                    update_message(bot, query.message, "Available categories", keyboard).await?;
                }
            },
        }
    };

    Ok(())
}

async fn run() -> Result<(), Box<dyn Error>> {
    let bot = Bot::from_env().auto_send();

    Dispatcher::new(bot)
        .messages_handler(|rx: DispatcherHandlerRx<AutoSend<Bot>, Message>| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, |cx| async move {
                message_handler(cx).await.log_on_error().await;
            })
        })
        .callback_queries_handler(|rx: DispatcherHandlerRx<AutoSend<Bot>, CallbackQuery>| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, |cx| async move {
                callback_handler(cx).await.log_on_error().await;
            })
        })
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
