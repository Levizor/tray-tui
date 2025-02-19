use ratatui::layout::Rect;
use std::{
    cell::RefCell,
    collections::HashMap,
    error,
    sync::{Arc, Mutex, MutexGuard},
};
use system_tray::{
    client::{Client, Event},
    item::StatusNotifierItem,
    menu::TrayMenu,
};

use tokio::sync::broadcast::Receiver;

use crate::wrappers::KeyRect;

pub type BoxStack = Vec<(i32, Rect)>;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub enum BoxStackKey {
    StatusNotifierItemId(String),
    MenuId(i32),
}

pub struct AppState {}

/// Application.
#[derive(Debug)]
pub struct App {
    pub client: Client,
    pub running: bool,
    pub tray_rx: Mutex<Receiver<Event>>,
    items: Arc<Mutex<HashMap<String, (StatusNotifierItem, Option<TrayMenu>)>>>,
    pub keys: Vec<KeyRect>, // for the StatusNotifierItem
    pub focused_key: Option<String>,
    pub box_stack: RefCell<Vec<(i32, Rect)>>, // for the tray menus
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(client: Client) -> Self {
        Self {
            running: true,
            tray_rx: Mutex::new(client.subscribe()),
            items: client.items(),
            keys: Vec::new(),
            box_stack: RefCell::default(),
            client,
            focused_key: None,
        }
    }

    /// Updating keys vector (for tracking of the current focus)
    pub fn update(&mut self, update: Event) {
        log::info!("UPDATE: {:?}", update);
        //log::debug!("Items now: {:?}", self.items);
        let mut buffer: Vec<KeyRect> = Vec::new();
        if let Some(items) = self.get_items() {
            buffer = items.keys().cloned().map(KeyRect::new).collect();
            buffer.sort();
        }
        self.keys = buffer;
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn get_items(
        &self,
    ) -> Option<MutexGuard<HashMap<String, (StatusNotifierItem, Option<TrayMenu>)>>> {
        match self.items.lock() {
            Ok(items) => Some(items),
            Err(_) => None,
        }
    }
}
