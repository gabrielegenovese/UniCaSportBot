use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};
use std::{fmt, fs, sync::Mutex};
use teloxide::{prelude::*, types::Chat, utils::command::BotCommands};
use tokio::time::{Duration, sleep};

const SUB_FILE: &str = "subs.json";
const EVENTS_FILE: &str = "events.json";
const UNICA_SPORT_URL: &str = "https://sport.univ-cotedazur.fr/fr/";

macro_rules! debugln {
    ($($arg:tt)*) => {
        if std::env::var("DEBUG").map(|v| v == "true").unwrap_or(false) {
            println!($($arg)*);
        }
    }
}

#[derive(PartialEq, Clone, serde::Serialize, serde::Deserialize)]
struct Event {
    title: String,
    date: String,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.title, self.date)
    }
}

static SUB_LIST: Lazy<Mutex<Vec<ChatId>>> = Lazy::new(|| {
    let path = sub_file();
    let data = match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => {
            let empty: Vec<ChatId> = Vec::new();
            let _ = fs::write(&path, serde_json::to_string(&empty).unwrap());
            empty
        }
    };
    Mutex::new(data)
});

static EVENT_LIST: Lazy<Mutex<Vec<Event>>> = Lazy::new(|| {
    let path = events_file();
    let data = match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => {
            let empty: Vec<Event> = Vec::new();
            let _ = fs::write(&path, serde_json::to_string(&empty).unwrap());
            empty
        }
    };
    Mutex::new(data)
});

lazy_static::lazy_static! {
    static ref EVENT_SELECTOR: Selector = Selector::parse("div.event").unwrap();
    static ref TITLE_SELECTOR: Selector = Selector::parse("div.event-info > h3.event-title").unwrap();
    static ref DATE_SELECTOR: Selector = Selector::parse("div.event-img > p.event-date").unwrap();
    static ref SPACE_REGEX: Regex = Regex::new(r"\s+").unwrap();
}

fn save_subs(data: &[ChatId]) {
    let _ = fs::write(sub_file(), serde_json::to_string(data).unwrap());
}

fn save_events(data: &[Event]) {
    let _ = fs::write(events_file(), serde_json::to_string(data).unwrap());
}

fn add_sub(chat: &Chat) {
    let mut list = SUB_LIST.lock().unwrap();
    if !list.contains(&chat.id) {
        list.push(chat.id);
        save_subs(&list);
    }
}

fn remove_sub(chat_id: ChatId) {
    let mut list = SUB_LIST.lock().unwrap();
    if let Some(_) = list.iter().position(|&id| id == chat_id) {
        list.retain(|&id| id != chat_id);
        save_subs(&list);
    }
}

fn add_event(event: Event) {
    let mut list = EVENT_LIST.lock().unwrap();
    list.push(event);
    save_events(&list);
}

fn remove_items(to_remove: Vec<Event>) {
    let mut list = EVENT_LIST.lock().unwrap();
    list.retain(|item| !to_remove.contains(item));
    save_events(&list);
}

fn clean_old_events(current: &[Event]) {
    let existing = EVENT_LIST.lock().unwrap().clone();
    let outdated: Vec<_> = existing
        .into_iter()
        .filter(|x| !current.contains(x))
        .collect();
    remove_items(outdated);
}

async fn check_website() -> Result<Vec<Event>, String> {
    let res = reqwest::get(UNICA_SPORT_URL)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let mut current: Vec<Event> = Vec::new();

    let document = Html::parse_document(&res);

    for event in document.select(&EVENT_SELECTOR) {
        let title_el = event.select(&TITLE_SELECTOR).next().unwrap();
        let date_el = event.select(&DATE_SELECTOR).next().unwrap();
        let title = title_el
            .text()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();
        let raw_date = date_el
            .text()
            .collect::<Vec<_>>()
            .join(" ")
            .replace('\n', " ");
        let date = SPACE_REGEX.replace_all(&raw_date, " ").trim().to_string();
        current.push(Event { title, date });
    }

    clean_old_events(&current);

    let new_events: Vec<_> = current
        .into_iter()
        .filter(|e| !EVENT_LIST.lock().unwrap().contains(e))
        .collect();

    if new_events.is_empty() {
        Err("Nothing new".to_string())
    } else {
        Ok(new_events)
    }
}

fn file_path(name: &str) -> String {
    let dir = std::env::var("UNICABOT_DATA_DIR").unwrap_or_else(|_| ".".to_string());
    format!("{}/{}", dir, name)
}

fn sub_file() -> String {
    file_path(SUB_FILE)
}

fn events_file() -> String {
    file_path(EVENTS_FILE)
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting UniCa Sport bot...");

    let bot = Bot::from_env();
    let repl_bot = bot.clone();
    let loop_bot = bot.clone();

    let repl_handle = tokio::spawn(async move {
        Command::repl(repl_bot, answer).await;
    });

    let checker_handle = tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(10)).await;
            match check_website().await {
                Ok(new_events) => {
                    for event in new_events {
                        add_event(event.clone());
                        debugln!("Added event {}", event);
                        let subs = SUB_LIST.lock().unwrap().clone();
                        for id in subs {
                            let _ = loop_bot
                                .send_message(id, format!("New event: {}", event))
                                .await;
                        }
                    }
                }
                Err(e) => log::info!("{}", e),
            }
            sleep(Duration::from_secs(600)).await;
        }
    });

    let _ = tokio::join!(repl_handle, checker_handle);
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
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
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Subscribe => {
            add_sub(&msg.chat);
            debugln!("Subscribed user {:?}", msg.chat.id);
            bot.send_message(
                msg.chat.id,
                "You've been subscribed to UniCa Sport event notifications.",
            )
            .await?
        }
        Command::Unsubscribe => {
            remove_sub(msg.chat.id);
            bot.send_message(msg.chat.id, "You've been unsubscribed from notifications.")
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
        Command::AmISubscribed => {
            let list = SUB_LIST.lock().unwrap().clone();
            let msg_text = if list.contains(&msg.chat.id) {
                "You are currently subscribed."
            } else {
                "You are not subscribed."
            };
            bot.send_message(msg.chat.id, msg_text).await?
        }
    };
    Ok(())
}
