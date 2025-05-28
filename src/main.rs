use once_cell::sync::Lazy;
use scraper::{Html, Selector};
use std::fmt;
use std::sync::Mutex;
use teloxide::{prelude::*, types::Chat, utils::command::BotCommands};
use tokio::time::{Duration, sleep};
use regex::Regex;

#[derive(PartialEq, Clone)]
struct Event {
    title: String,
    date: String,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Event: {} ({})", self.title, self.date)
    }
}

static SUB_LIST: Lazy<Mutex<Vec<ChatId>>> = Lazy::new(|| Mutex::new(Vec::new()));
static EVENT_LIST: Lazy<Mutex<Vec<Event>>> = Lazy::new(|| Mutex::new(Vec::new()));

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

fn add_event(item: Event) {
    let mut list = EVENT_LIST.lock().unwrap();
    list.push(item);
}

fn remove_items(to_remove: Vec<Event>) {
    let mut list = EVENT_LIST.lock().unwrap();
    list.retain(|item| !to_remove.contains(item));
}

fn clean_old_events(curr_event_list: &Vec<Event>) {
    let list = EVENT_LIST.lock().unwrap().clone();
    let diff: Vec<_> = list
        .into_iter()
        .filter(|x| !curr_event_list.contains(x))
        .collect();

    remove_items(diff);
}

async fn check_website() -> Result<Vec<Event>, String> {
    let url = "https://sport.univ-cotedazur.fr/fr/";
    let res = reqwest::get(url).await.unwrap().text().await.unwrap();

    let mut curr_event_list: Vec<Event> = Vec::new();

    let document = Html::parse_document(&res);
    let event_selector = Selector::parse("div.event").unwrap();
    let title_selector = Selector::parse("div.event-info > h3.event-title").unwrap();
    let date_selector = Selector::parse("div.event-img > p.event-date").unwrap();

    for event in document.select(&event_selector) {
        let title_el = event.select(&title_selector).next().unwrap();
        let date_el = event.select(&date_selector).next().unwrap();
        let title = title_el.text().collect::<Vec<_>>().join(" ");
        let date_unclean = date_el.text().collect::<Vec<_>>().join(" ").replace('\n', " ");
        let re = Regex::new(r"\s+").unwrap();
        let date = re.replace_all(&date_unclean, " ").to_string();
        let e = Event {
            title: title.clone(),
            date: date.clone(),
        };
        curr_event_list.push(e);
        log::info!("Found Event: {} - Date: {}.", title.trim(), date.trim());
        println!("Event Title: {} - Date: {}", title.trim(), date.trim());
    }

    clean_old_events(&curr_event_list);

    let diff: Vec<_> = curr_event_list
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

                        for id in subs {
                            log::info!("New {}", e);
                            println!("New {}", e);
                            let _ = bot_for_loop.send_message(id, format!("New {}", e)).await;
                        }
                    }
                }
                Err(e) => println!("{}", e),
            }

            sleep(Duration::from_secs(600)).await; // check every 10m
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
    #[command(description = "Display this text.")]
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
            bot.send_message(msg.chat.id, 
                format!("You've been successfully subscribed to the list, I'll send you new events when added."))
                .await?
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
