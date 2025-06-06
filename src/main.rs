mod commands;
mod constants;
mod events;
mod scraper;
mod subs;

use commands::reply_process;
use dotenv::dotenv;
use scraper::scraper_process;
use teloxide::prelude::*;

pub fn debug_is(what: bool) -> bool {
    std::env::var("DEBUG")
        .map(|v| v == what.to_string())
        .unwrap_or(false)
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting UniCa Sport bot...");

    let bot = Bot::from_env();
    let repl_handle = reply_process(bot.clone());
    let checker_handle = scraper_process(bot);
    let _ = tokio::join!(repl_handle, checker_handle);
}
