use crate::constants::SUB_FILE;
use once_cell::sync::Lazy;
use std::{fs, sync::Mutex};
use teloxide::types::ChatId;

pub static SUB_LIST: Lazy<Mutex<Vec<ChatId>>> = Lazy::new(|| {
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

pub fn add_sub(chat_id: ChatId) {
    let mut list = SUB_LIST.lock().unwrap();
    if !list.contains(&chat_id) {
        list.push(chat_id);
        save_subs_file(&list);
    }
}

pub fn remove_sub(chat_id: ChatId) {
    let mut list = SUB_LIST.lock().unwrap();
    list.retain(|&id| id != chat_id);
    save_subs_file(&list);
}

pub fn file_path(name: &str) -> String {
    let dir = std::env::var("UNICABOT_DATA_DIR").unwrap_or_else(|_| ".".to_string());
    format!("{}/{}", dir, name)
}

pub fn sub_file() -> String {
    file_path(SUB_FILE)
}

fn save_subs_file(data: &[ChatId]) {
    let _ = fs::write(sub_file(), serde_json::to_string(data).unwrap());
}
