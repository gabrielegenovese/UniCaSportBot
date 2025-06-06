use crate::constants::debug_is;
use crate::debugln;
use crate::events::{Event, add_event, get_new_events};
use crate::subs::SUB_LIST;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use tokio::time::{Duration, sleep};

pub fn scraper_process(bot: Bot) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(5)).await;
            match get_new_events().await {
                Ok(new_events) => send_notifications(new_events, &bot).await,
                Err(e) => log::info!("{}", e),
            }
            sleep(if debug_is(true) {
                Duration::from_secs(10)
            } else {
                Duration::from_secs(600)
            })
            .await;
        }
    })
}

async fn send_notifications(new_events: Vec<Event>, bot: &Bot) {
    for event in new_events {
        add_event(event.clone());
        debugln!("Added event {}", event);
        let subs = SUB_LIST.lock().unwrap().clone();
        for id in subs {
            let _ = bot
                .send_message(id, format!("New event: {}", event))
                .parse_mode(ParseMode::Html)
                .await;
        }
    }
}
