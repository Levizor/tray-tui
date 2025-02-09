use std::{collections::HashMap, error, sync::Arc, sync::Mutex};
use system_tray::{
    client::{Client, Event},
    item::StatusNotifierItem,
    menu::TrayMenu,
};

use tokio::sync::broadcast::Receiver;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub tray_rx: Mutex<Receiver<Event>>,
    client: Client,
    items: Arc<Mutex<HashMap<String, (StatusNotifierItem, Option<TrayMenu>)>>>,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(client: Client) -> Self {
        Self {
            running: true,
            tray_rx: Mutex::new(client.subscribe()),
            items: client.items(),
            client,
        }
    }

    pub fn feed(&mut self, update: Event) -> Result<(), ()> {
        log::info!("{:?}", self.items.lock().unwrap());
        //match update {
        //    Event::Add(item, status_notifier_item) => todo!(),
        //    Event::Update(item, update_event) => todo!(),
        //    Event::Remove(item) => {}
        //}
        Ok(())
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn get_items(&self) -> HashMap<String, (StatusNotifierItem, Option<TrayMenu>)> {
        self.items.lock().unwrap().clone()
    }
}
