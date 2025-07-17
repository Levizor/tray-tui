use indexmap::IndexMap;
use ratatui::layout::{Position, Rect};
use std::{
    cell::{Ref, RefMut},
    collections::{HashMap, HashSet},
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

use crate::wrappers::{FindMenuByUsize, Id, SniState};
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
    // index of currently focused sni item
    pub focused_sni: Option<usize>,
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
            focused_sni: None,
        }
    }

    /// Updating states
    pub fn update(&mut self) {
        //log::debug!("ITEMS NOW: {:?}", self.get_items().unwrap());
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
        // TODO sorting
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
        let index = self.get_focused_sni_index()?.clone();
        self.sni_states.get_index_mut(index)?.1.focused = false;
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
        let (_, val) = self.sni_states.get_index_mut(new_index)?;
        val.focused = true;
        self.focused_sni = Some(new_index);

        Some(())
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
                log::debug!("{:?}", activate_request);
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
