use crate::constants::*;
use crate::debugln;
use crate::events::{EVENT_LIST, Event};
use crate::subs::{SUB_LIST, add_sub, remove_sub};
use teloxide::types::ParseMode;
use teloxide::utils::command::BotCommands;
use teloxide::{RequestError, prelude::*};

pub fn reply_process(repl_bot: Bot) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        Command::repl(repl_bot, answer).await;
    })
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "Start this bot and display a welcome message.")]
    Start,
    #[command(description = "Display this help text.")]
    Help,
    #[command(description = "Subscribe to event notifications.")]
    Subscribe,
    #[command(description = "Unsubscribe from notifications.")]
    Unsubscribe,
    #[command(description = "List known events.")]
    Events,
    #[command(description = "Check if you are subscribed.")]
    AmISubscribed,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Start => manage_start_cmd(bot, msg).await?,
        Command::Help => manage_help_cmd(bot, msg).await?,
        Command::Subscribe => manage_sub_cmd(bot, msg).await?,
        Command::Unsubscribe => manage_unsub_cmd(bot, msg).await?,
        Command::Events => manage_event_cmd(bot, msg).await?,
        Command::AmISubscribed => manage_amisub_cmd(bot, msg).await?,
    };
    Ok(())
}

async fn manage_start_cmd(bot: Bot, msg: Message) -> Result<Message, RequestError> {
    bot.send_message(msg.chat.id, WELCOME_MSG).await
}

async fn manage_help_cmd(bot: Bot, msg: Message) -> Result<Message, RequestError> {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await
}

async fn manage_sub_cmd(bot: Bot, msg: Message) -> Result<Message, RequestError> {
    add_sub(msg.chat.id);
    debugln!("Subscribed user {:?}", msg.chat.id);
    bot.send_message(msg.chat.id, SUB_MSG).await
}

async fn manage_unsub_cmd(bot: Bot, msg: Message) -> Result<Message, RequestError> {
    remove_sub(msg.chat.id);
    bot.send_message(msg.chat.id, UNSUB_MSG).await
}

fn format_events_msg(events: Vec<Event>) -> String {
    format!(
        "Current events:\n{}",
        events
            .iter()
            .map(|e| format!("• {}", e))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

async fn manage_event_cmd(bot: Bot, msg: Message) -> Result<Message, RequestError> {
    let events = EVENT_LIST.lock().unwrap().clone();
    let text = if events.is_empty() {
        NOEVENTS.to_string()
    } else {
        format_events_msg(events)
    };
    bot.send_message(msg.chat.id, text)
        .parse_mode(ParseMode::Html)
        .await
}

async fn manage_amisub_cmd(bot: Bot, msg: Message) -> Result<Message, RequestError> {
    let list = SUB_LIST.lock().unwrap().clone();
    let msg_text = if list.contains(&msg.chat.id) {
        IAMSUB_MSG
    } else {
        IAMNOTSUB_MSG
    };
    bot.send_message(msg.chat.id, msg_text).await
}
