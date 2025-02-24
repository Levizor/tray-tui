use indexmap::IndexMap;
use ratatui::layout::{Position, Rect};
use std::{
    cell::{Ref, RefMut},
    collections::{HashMap, HashSet},
    error,
    sync::{Arc, Mutex, MutexGuard},
};
use system_tray::{
    client::{Client, Event},
    item::StatusNotifierItem,
    menu::TrayMenu,
};
use tui_tree_widget::TreeState;

use tokio::sync::broadcast::Receiver;

use crate::wrappers::SniState;
use crate::Config;

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
    pub running: bool,
    /// Config
    pub config: Config,
    /// system-tray client
    pub client: Client,
    /// states saved for each [StatusNotifierItem] and their [TrayMenu]
    pub sni_states: IndexMap<String, SniState>, // for the StatusNotifierItem
    // index of currently focused sni item
    pub focused_sni: Option<usize>,
    /// items map from system-tray
    items: Arc<Mutex<HashMap<String, (StatusNotifierItem, Option<TrayMenu>)>>>,
    pub tray_rx: Mutex<Receiver<Event>>,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(client: Client, config: Config) -> Self {
        Self {
            running: true,
            config,
            tray_rx: Mutex::new(client.subscribe()),
            items: client.items(),
            sni_states: IndexMap::default(),
            client,
            focused_sni: None,
        }
    }

    /// Updating keys vector (for tracking of the current focus)
    pub fn update(&mut self, update: Event) {
        log::info!("UPDATE: {:?}", update);
        log::info!("ITEMS NOW: {:?}", self.get_items().unwrap());
        let mut buffer: HashSet<String> = HashSet::default();
        if let Some(items) = self.get_items() {
            buffer = items.keys().cloned().collect();
        }
        if buffer.len() == self.sni_states.len() {
            return;
        }
        for key in &buffer {
            self.sni_states
                .entry(key.to_string())
                .or_insert_with(|| SniState::new());
        }
        self.sni_states.retain(|key, _| buffer.contains(key));
        if let Some(index) = self.get_focused_sni_index() {
            if let Some((_, v)) = self.sni_states.get_index_mut(*index) {
                v.set_focused(true);
                return;
            }
        }
        if let Some((_, v)) = self.sni_states.first_mut() {
            v.set_focused(true);
            self.focused_sni = Some(0);
        }
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

    pub fn get_focused_sni_key(&self) -> Option<&String> {
        self.sni_states
            .get_index(*self.get_focused_sni_index()?)
            .map(|(k, _)| k)
    }

    pub fn get_focused_sni_index(&self) -> Option<&usize> {
        self.focused_sni.as_ref()
    }

    pub fn get_focused_sni_state(&self) -> Option<&SniState> {
        if let Some(key) = self.focused_sni {
            let (_, v) = self.sni_states.get_index(key)?;
            return Some(v);
        }
        None
    }

    pub fn get_focused_sni_state_mut(&mut self) -> Option<&mut SniState> {
        if let Some(key) = self.focused_sni {
            let (_, v) = self.sni_states.get_index_mut(key)?;
            return Some(v);
        }
        None
    }

    pub fn get_focused_sni_key_by_position(&mut self, pos: Position) -> Option<String> {
        self.sni_states
            .iter()
            .find(|(_, v)| v.rect.contains(pos))
            .map(|(k, _)| k.to_string())
    }

    pub fn get_focused_tree_state(&self) -> Option<Ref<TreeState<i32>>> {
        self.get_focused_sni_state()
            .map(|sni| sni.tree_state.borrow())
    }

    pub fn get_focused_tree_state_mut(&self) -> Option<RefMut<TreeState<i32>>> {
        self.get_focused_sni_state()
            .map(|sni| sni.tree_state.borrow_mut())
    }

    pub fn move_focus_to_right(&mut self) -> Option<()> {
        let len = self.sni_states.len();
        if len <= 1 {
            return Some(());
        }

        let index = self.get_focused_sni_index()?.clone();
        self.sni_states.get_index_mut(index)?.1.focused = false;

        let new_index = (index + 1) % len;

        let (_, val) = self.sni_states.get_index_mut(new_index)?;
        val.focused = true;
        self.focused_sni = Some(new_index);

        Some(())
    }

    pub fn move_focus_to_left(&mut self) -> Option<()> {
        let len = self.sni_states.len();
        if len <= 1 {
            return Some(());
        }

        let index = self.get_focused_sni_index()?.clone();
        self.sni_states.get_index_mut(index)?.1.focused = false;

        let new_index = match index {
            0 => len - 1,
            _ => index - 1,
        };

        let (_, val) = self.sni_states.get_index_mut(new_index)?;
        val.focused = true;
        self.focused_sni = Some(new_index);

        Some(())
    }
}
