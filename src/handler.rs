use crate::{
    app::{App, AppResult},
    wrappers::KeyRect,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent};
use ratatui::layout::Position;
use system_tray::{
    client::ActivateRequest,
    menu::{MenuItem, TrayMenu},
};
use tui_tree_widget::Tree;

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}

fn get_pos(mouse_event: MouseEvent) -> Position {
    Position::new(mouse_event.column, mouse_event.row)
}

fn get_focused_status_notifier_keyrect_mut(app: &mut App, pos: Position) -> Option<&mut KeyRect> {
    app.keys.iter_mut().find(|k| k.rect.contains(pos))
}

fn get_focused_status_notifier_keyrect(app: &App, pos: Position) -> Option<&KeyRect> {
    app.keys.iter().find(|k| k.rect.contains(pos))
}

trait FindById {
    fn find_by_id(&self, id: i32) -> Option<&MenuItem>;
}

fn recursive_find(item: &MenuItem, id: i32) -> Option<&MenuItem> {
    if item.id == id {
        return Some(&item);
    }
    for item in &item.submenu {
        if let Some(item) = recursive_find(item, id) {
            return Some(item);
        }
        log::info!("Iterating");
    }

    None
}

impl FindById for TrayMenu {
    fn find_by_id(&self, id: i32) -> Option<&MenuItem> {
        for item in &self.submenus {
            if let Some(item) = recursive_find(item, id) {
                return Some(item);
            }
        }

        None
    }
}
async fn handle_click(mouse_event: MouseEvent, app: &App) -> Option<()> {
    let pos = get_pos(mouse_event);

    let mut tree_state = app.menu_tree_state.borrow_mut();
    let opt_id = tree_state.rendered_at(pos);
    let id = opt_id.map(|vec| vec[0])?;
    let focused_sni = get_focused_status_notifier_keyrect(app, pos)?;
    let map = app.get_items()?;
    let (sni, menu) = map.get(&focused_sni.key)?;
    let menu = match menu {
        Some(menu) => menu,
        None => return None,
    };

    let item = menu.find_by_id(id)?;

    if item.submenu.is_empty() {
        if let Some(path) = &sni.menu {
            let _ = app
                .client
                .activate(ActivateRequest::MenuItem {
                    address: focused_sni.key.clone(),
                    menu_path: path.to_string(),
                    submenu_id: id,
                })
                .await;
        }
    } else {
        tree_state.toggle(vec![id]);
    }
    Some(())
}

async fn handle_move(mouse_event: MouseEvent, app: &mut App) {
    let pos = get_pos(mouse_event);
    if let Some(key) = &app.focused_key {
        if let Some(keyrect) = app.keys.iter_mut().find(|k| k.key.eq(key)) {
            if keyrect.rect.contains(pos) {
                keyrect.set_focused(true);
                return;
            } else {
                keyrect.set_focused(false);
                let _ = app.menu_tree_state.take();
            }
        }
    }

    if let Some(k) = get_focused_status_notifier_keyrect_mut(app, pos) {
        k.set_focused(true);
        app.focused_key = Some(k.key.clone());
    } else {
        app.focused_key = None;
    }
}

pub async fn handle_mouse_event(mouse_event: MouseEvent, app: &mut App) -> AppResult<()> {
    match mouse_event.kind {
        crossterm::event::MouseEventKind::Down(MouseButton::Left) => {
            handle_click(mouse_event, app).await
        }
        crossterm::event::MouseEventKind::Down(MouseButton::Right) => {}
        crossterm::event::MouseEventKind::Down(MouseButton::Middle) => {}
        crossterm::event::MouseEventKind::Moved => handle_move(mouse_event, app).await,
        _ => {}
    }
    Ok(())
}
