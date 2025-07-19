use indexmap::IndexMap;
use ratatui::layout::{Position, Rect};
use std::{
    cell::{Ref, RefMut},
    collections::HashMap,
    error,
    sync::{Arc, Mutex, MutexGuard},
};
use system_tray::client::ActivateRequest;
use system_tray::{
    client::{Client, Event},
    item::StatusNotifierItem,
    menu::TrayMenu,
};
use tui_tree_widget::TreeState;

use tokio::sync::broadcast::Receiver;

use crate::wrappers::{FindMenuByUsize, GetTitle, Id, SniState};
use crate::Config;

pub type BoxStack = Vec<(i32, Rect)>;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

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
    //  currently focused sni item info
    pub focused_sni_index: usize,
    pub focused_sni_key: String,
    /// items map from system-tray
    pub items: Arc<Mutex<HashMap<String, (StatusNotifierItem, Option<TrayMenu>)>>>,
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
            focused_sni_index: 0,
            focused_sni_key: String::default(),
        }
    }

    /// Updating states
    pub fn update(&mut self) {
        // sync key to index
        if let Some(key) = self.get_focused_sni_key() {
            self.focused_sni_key = key.to_owned();
        }

        // create a buffer for items keys and their titles(for sorting)
        let mut buffer = IndexMap::new();
        if let Some(items) = self.get_items() {
            buffer = items
                .iter()
                .map(|(k, v)| (k.to_owned(), v.0.get_title().to_owned()))
                .collect();
        }

        // Add sni states if there are in new items
        for (key, _) in &buffer {
            self.sni_states
                .entry(key.to_owned())
                .or_insert_with(|| SniState::new());
        }

        // Remove states that aren't in new items
        self.sni_states.retain(|key, _| buffer.contains_key(key));

        // Sort by titles
        if self.config.sorting {
            self.sni_states
                .sort_by(|k1, _, k2, _| buffer[k1].cmp(&buffer[k2]));
        }

        // sync index to key back
        if let Some(index) = self.sni_states.get_index_of(&self.focused_sni_key) {
            self.focused_sni_index = index;
        }

        // Synchronize focus
        self.sync_focus();
    }

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
            .get_index(*self.get_focused_sni_index())
            .map(|(k, _)| k)
    }

    pub fn get_focused_sni_index(&self) -> &usize {
        &self.focused_sni_index
    }

    pub fn get_focused_sni_state(&self) -> Option<&SniState> {
        let (_, v) = self.sni_states.get_index(self.focused_sni_index)?;
        return Some(v);
    }

    pub fn get_focused_sni_state_mut(&mut self) -> Option<&mut SniState> {
        let (_, v) = self.sni_states.get_index_mut(self.focused_sni_index)?;
        return Some(v);
    }

    pub fn get_focused_sni_key_by_position(&mut self, pos: Position) -> Option<String> {
        self.sni_states
            .iter()
            .find(|(_, v)| v.rect.contains(pos))
            .map(|(k, _)| k.to_string())
    }

    pub fn get_focused_tree_state(&self) -> Option<Ref<TreeState<Id>>> {
        self.get_focused_sni_state()
            .map(|sni| sni.tree_state.borrow())
    }

    pub fn get_focused_tree_state_mut(&self) -> Option<RefMut<TreeState<Id>>> {
        self.get_focused_sni_state()
            .map(|sni| sni.tree_state.borrow_mut())
    }

    pub fn move_focus(&mut self, direction: FocusDirection) -> Option<()> {
        let len = self.sni_states.len();
        if len <= 1 {
            return Some(());
        }
        let index = self.focused_sni_index;
        let columns = self.config.columns;

        let new_index = match direction {
            FocusDirection::Down => {
                if index == len - 1 {
                    0
                } else {
                    (index + columns).min(len - 1)
                }
            }
            FocusDirection::Up => index.checked_sub(columns).unwrap_or_else(|| {
                let mut i = index;
                while i + columns < len {
                    i += columns;
                }
                i
            }),
            FocusDirection::Right => {
                if index + 1 < len {
                    index + 1
                } else {
                    0
                }
            }
            FocusDirection::Left => index.checked_sub(1).unwrap_or(len - 1),
        };

        self.focused_sni_index = new_index;
        self.focused_sni_key = self
            .sni_states
            .get_index(self.focused_sni_index)
            .unwrap()
            .0
            .clone();
        self.sni_states.get_index_mut(index)?.1.set_focused(false);
        self.sync_focus();

        Some(())
    }

    pub fn sync_focus(&mut self) {
        if let Some(val) = self.sni_states.get_mut(&self.focused_sni_key) {
            val.set_focused(true);
        }
    }

    pub async fn activate_menu_item(
        &self,
        ids: &[Id],
        tree_state: &mut TreeState<Id>,
    ) -> Option<()> {
        let sni_key = self.get_focused_sni_key()?;
        let map = self.get_items()?;
        let (sni, menu) = map.get(sni_key)?;
        let menu = match menu {
            Some(menu) => menu,
            None => return None,
        };

        let item = menu.find_menu_by_usize(ids)?;

        if item.submenu.is_empty() {
            if let Some(path) = &sni.menu {
                let activate_request = ActivateRequest::MenuItem {
                    address: sni_key.to_string(),
                    menu_path: path.to_string(),
                    submenu_id: item.id,
                };
                let _ = self.client.activate(activate_request).await;

                let _ = self
                    .client
                    .about_to_show_menuitem(sni_key.to_string(), path.to_string(), 0)
                    .await;
            }
        } else {
            tree_state.toggle(ids.to_vec());
        }

        Some(())
    }
}
pub enum FocusDirection {
    Down,
    Up,
    Right,
    Left,
}
