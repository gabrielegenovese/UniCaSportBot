// MISCs
pub const SUB_FILE: &str = "subs.json";
pub const EVENTS_FILE: &str = "events.json";
pub const UNICA_SPORT_URL: &str = "https://sport.univ-cotedazur.fr/fr/";

// MSGs
pub const WELCOME_MSG: &str = "👋 Welcome to UniCa Sport Bot!\n\n\
Use /subscribe to receive notifications about new UniCa's sport events.\
Need help? Type /help to see all available commands.";
pub const SUB_MSG: &str = "You've been subscribed to UniCa Sport event notifications.";
pub const UNSUB_MSG: &str = "You've been unsubscribed from notifications.";
pub const IAMSUB_MSG: &str = "You are currently subscribed.";
pub const IAMNOTSUB_MSG: &str = "You are not subscribed.";
pub const NOEVENTS: &str = "No known events yet.";

pub fn debug_is(what: bool) -> bool {
    std::env::var("DEBUG")
        .map(|v| v == what.to_string())
        .unwrap_or(false)
}

#[macro_export]
macro_rules! debugln {
    ($($arg:tt)*) => {
        if debug_is(true) {
            println!($($arg)*);
        }
    }
}
