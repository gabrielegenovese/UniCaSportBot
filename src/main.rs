use once_cell::sync::Lazy;
use scraper::{Html, Selector};
use std::sync::Mutex;
use teloxide::{prelude::*, types::Chat, utils::command::BotCommands};
use tokio::time::{Duration, sleep};

static SUB_LIST: Lazy<Mutex<Vec<ChatId>>> = Lazy::new(|| Mutex::new(Vec::new()));
static EVENT_LIST: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

fn add_sub(item: &Chat) {
    let mut list = SUB_LIST.lock().unwrap();
    if !list.contains(&item.id) {
        list.push(item.id);
    }
}

fn remove_sub(chat_id: ChatId) {
    let mut list = SUB_LIST.lock().unwrap();
    list.retain(|&id| id != chat_id);
}

fn add_event(item: String) {
    let mut list = EVENT_LIST.lock().unwrap();
    list.push(item);
}

async fn check_website() -> Result<Vec<String>, String> {
    let url = "https://sport.univ-cotedazur.fr/fr/";
    let res = reqwest::get(url).await.unwrap().text().await.unwrap();

    let mut ev_l: Vec<String> = Vec::new();

    let document = Html::parse_document(&res);
    let event_selector = Selector::parse("div.event").unwrap();
    let title_selector = Selector::parse("div.event-info > h3.event-title").unwrap();

    for event in document.select(&event_selector) {
        if let Some(title_el) = event.select(&title_selector).next() {
            let title = title_el.text().collect::<Vec<_>>().join(" ");
            ev_l.push(title.trim().to_string());
            println!("Event Title: {}", title.trim());
        }
    }

    let diff: Vec<_> = ev_l
        .into_iter()
        .filter(|x| !EVENT_LIST.lock().unwrap().contains(x))
        .collect();

    if diff.is_empty() {
        Err("Nothing new".to_string())
    } else {
        Ok(diff)
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot = Bot::from_env();

    let bot_for_repl = bot.clone();
    let bot_for_loop = bot.clone();

    // Spawn the bot command handler
    let repl_handle = tokio::spawn(async move {
        Command::repl(bot_for_repl, answer).await;
    });

    // Spawn the periodic check loop
    let checker_handle = tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(10)).await; // wait 10 sec
            match check_website().await {
                Ok(new_events) => {
                    for e in new_events {
                        add_event(e.clone());
                        let subs = SUB_LIST.lock().unwrap().clone();
                        // println!("Adding and sending");

                        for id in subs {
                            // println!("New event: {e}, sent to {id}");
                            let _ = bot_for_loop
                                .send_message(id, format!("New event: {}", e))
                                .await;
                        }
                    }
                }
                Err(e) => println!("{}", e),
            }

            sleep(Duration::from_secs(10)).await; // check every 10m
        }
    });

    // Wait for both tasks (actually repl never ends)
    let _ = tokio::join!(repl_handle, checker_handle);
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "Subscribe to receive update about the events of UniCa Sport.")]
    Subscribe,
    #[command(description = "Unsubscribe from event notifications.")]
    Unsubscribe,
    #[command(description = "Show current known events.")]
    Events,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Subscribe => {
            add_sub(&msg.chat);
            bot.send_message(msg.chat.id, format!("You've been successfully subscribed to the list, I'll send you new events when added.")).await?
        }
        Command::Unsubscribe => {
            remove_sub(msg.chat.id);
            bot.send_message(
                msg.chat.id,
                "You've been unsubscribed from event notifications.",
            )
            .await?
        }
        Command::Events => {
            let events = EVENT_LIST.lock().unwrap().clone();
            if events.is_empty() {
                bot.send_message(msg.chat.id, "No known events yet.")
                    .await?
            } else {
                let list = events
                    .iter()
                    .map(|e| format!("• {}", e))
                    .collect::<Vec<_>>()
                    .join("\n");
                bot.send_message(msg.chat.id, format!("Current events:\n{}", list))
                    .await?
            }
        }
    };

    Ok(())
}
