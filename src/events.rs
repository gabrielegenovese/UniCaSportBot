use crate::constants::{EVENTS_FILE, UNICA_SPORT_URL};
use crate::subs::file_path;
use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use std::{fmt, fs, sync::Mutex};

// TYPES AND STATIC DATAS
#[derive(PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Event {
    title: String,
    date: String,
    link: String,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<a href='{}'>{}</a> ({})",
            self.link, self.title, self.date
        )
    }
}

pub static EVENT_LIST: Lazy<Mutex<Vec<Event>>> = Lazy::new(|| {
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
    static ref LINK_SELECTOR: Selector = Selector::parse("div.event-info > p.text-right > a.btn").unwrap();
    static ref SPACE_REGEX: Regex = Regex::new(r"\s+").unwrap();
}

// APIs

pub fn add_event(event: Event) {
    let mut list = EVENT_LIST.lock().unwrap();
    list.push(event);
    save_events(&list);
}

pub async fn get_new_events() -> Result<Vec<Event>, String> {
    let html = fetch_page().await?;
    let events = parse_events(&html);
    clean_old_events(&events);
    let new_events = filter_new_events(events);

    if new_events.is_empty() {
        Err("Nothing new".to_string())
    } else {
        Ok(new_events)
    }
}

// HELP FUNCTIONS

fn remove_items(to_remove: Vec<Event>) {
    let mut list = EVENT_LIST.lock().unwrap();
    list.retain(|item| !to_remove.contains(item));
    save_events(&list);
}

fn events_file() -> String {
    file_path(EVENTS_FILE)
}

fn save_events(data: &[Event]) {
    let _ = fs::write(events_file(), serde_json::to_string(data).unwrap());
}

fn clean_old_events(current: &[Event]) {
    let existing = EVENT_LIST.lock().unwrap().clone();
    let outdated: Vec<_> = existing
        .into_iter()
        .filter(|x| !current.contains(x))
        .collect();
    remove_items(outdated);
}

async fn fetch_page() -> Result<String, String> {
    reqwest::get(UNICA_SPORT_URL)
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())
}

fn parse_events(html: &str) -> Vec<Event> {
    let document = Html::parse_document(html);
    document
        .select(&EVENT_SELECTOR)
        .filter_map(parse_event)
        .collect()
}

fn parse_event(element: scraper::element_ref::ElementRef) -> Option<Event> {
    let title = get_event_title(element);
    if title.to_lowercase().contains("test") {
        return None;
    }
    let date = get_event_date(element);
    let link = get_event_link(element);
    Some(Event { title, date, link })
}

fn get_event_title(event: ElementRef<'_>) -> String {
    event
        .select(&TITLE_SELECTOR)
        .next()
        .unwrap()
        .text()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn get_event_date(event: ElementRef<'_>) -> String {
    let raw_date = event
        .select(&DATE_SELECTOR)
        .next()
        .unwrap()
        .text()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .replace('\n', " ");
    SPACE_REGEX.replace_all(&raw_date, " ").trim().to_string()
}

fn get_event_link(event: ElementRef<'_>) -> String {
    let half_link = event
        .select(&LINK_SELECTOR)
        .filter_map(|el| el.value().attr("href"))
        .map(|href| href.chars().skip(4).collect::<String>())
        .next()
        .unwrap_or_default();
    format!("{UNICA_SPORT_URL}{half_link}")
}

fn filter_new_events(current: Vec<Event>) -> Vec<Event> {
    current
        .into_iter()
        .filter(|e| !EVENT_LIST.lock().unwrap().contains(e))
        .collect()
}
