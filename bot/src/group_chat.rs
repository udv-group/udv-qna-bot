use crate::dptree;
use teloxide::prelude2::*;
use teloxide_core::adaptors::AutoSend;
use teloxide_core::types::{ChatKind, PublicChatKind, Update};
use teloxide_core::Bot;

pub async fn handle_group_chat(_bot: AutoSend<Bot>) -> anyhow::Result<()> {
    log::info!("GroupMessage");
    Ok(())
}

pub fn filter_group_chats(upd: Update) -> bool {
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

pub fn make_group_chat_branch() -> Handler<'static, DependencyMap, anyhow::Result<()>, DependencyMap>
{
    dptree::entry().branch(Update::filter_my_chat_member().endpoint(handle_group_chat))
}
