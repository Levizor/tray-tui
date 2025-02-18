use ratatui::layout::Rect;
use std::{collections::HashMap, error, sync::Arc, sync::Mutex, sync::MutexGuard};
use system_tray::{
    client::{Client, Event, UpdateEvent},
    item::StatusNotifierItem,
    menu::TrayMenu,
};

use tokio::sync::broadcast::Receiver;

use crate::wrappers::KeyRect;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub tray_rx: Mutex<Receiver<Event>>,
    items: Arc<Mutex<HashMap<String, (StatusNotifierItem, Option<TrayMenu>)>>>,
    pub keys: Vec<KeyRect>,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(client: Client) -> Self {
        Self {
            running: true,
            tray_rx: Mutex::new(client.subscribe()),
            items: client.items(),
            keys: Vec::new(),
        }
    }

    /// Updating keys vector (for tracking of the current focus)
    pub fn update(&mut self, update: Event) {
        log::info!("UPDATE: {:?}", update);
        log::debug!("Items now: {:?}", self.items);
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
