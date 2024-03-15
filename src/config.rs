use std::{cell::RefCell, rc::Rc};

use leptos::{expect_context, provide_context};

#[derive(Debug)]
pub struct Config {
    pub save_position: bool,
    pub cache_book: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            save_position: true,
            cache_book: false,
        }
    }
}

impl Config {
    pub fn save(&self) {
        let Ok(Some(storage)) = leptos::window().local_storage() else {
            return;
        };

        let config_string = format!("{}:{}", self.save_position, self.cache_book);
        let _ = storage.set_item("c", &config_string);
    }
}

fn load() -> Option<Config> {
    let Ok(Some(storage)) = leptos::window().local_storage() else {
        return None;
    };
    let Ok(Some(config_string)) = storage.get_item("c") else {
        return None;
    };

    let (save_position, cache_book) = config_string.split_once(':')?;
    Some(Config {
        save_position: save_position.parse::<bool>().ok()?,
        cache_book: cache_book.parse::<bool>().ok()?,
    })
}

pub fn init() -> Rc<RefCell<Config>> {
    let config = Rc::new(RefCell::new(load().unwrap_or_default()));
    provide_context(config.clone());
    config
}

pub fn get() -> Rc<RefCell<Config>> {
    expect_context::<Rc<RefCell<Config>>>()
}
