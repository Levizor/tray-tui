use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;
use system_tray::{
    client::ActivateRequest,
    menu::{MenuItem, TrayMenu},
};
use tui_tree_widget::TreeState;

/// Handles the key events and updates the state of [`App`].
pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
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
        KeyCode::Left | KeyCode::Char('h') => {
            app.move_focus_to_left();
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.move_focus_to_right();
        }
        _ => {}
    }
    let tree_state = app.get_focused_tree_state_mut();
    if tree_state.is_none() {
        return Ok(());
    }
    let mut tree_state = &mut tree_state.unwrap();
    match key_event.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if !tree_state.key_up() {
                tree_state.select_last();
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !tree_state.key_down() {
                tree_state.select_first();
            }
        }
        KeyCode::Enter => {
            let id = tree_state.selected().get(0).cloned();
            match id {
                Some(id) => {
                    let _ = activate_menu_item(id, app, &mut tree_state).await;
                }
                None => {
                    let _ = tree_state.select_first();
                }
            }
        }
        _ => {}
    }

    Ok(())
}

fn get_pos(mouse_event: MouseEvent) -> Position {
    Position::new(mouse_event.column, mouse_event.row)
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

async fn activate_menu_item(id: i32, app: &App, tree_state: &mut TreeState<i32>) -> Option<()> {
    let sni_key = app.get_focused_sni_key()?;
    let map = app.get_items()?;
    let (sni, menu) = map.get(sni_key)?;
    let menu = match menu {
        Some(menu) => menu,
        None => return None,
    };

    let item = menu.find_by_id(id)?;

    if item.submenu.is_empty() {
        if let Some(path) = &sni.menu {
            let activate_request = ActivateRequest::MenuItem {
                address: sni_key.to_string(),
                menu_path: path.to_string(),
                submenu_id: id,
            };
            log::debug!("{:?}", activate_request);
            let _ = app.client.activate(activate_request).await;

            let _ = app
                .client
                .about_to_show_menuitem(sni_key.to_string(), path.to_string(), 0)
                .await;
        }
    } else {
        tree_state.toggle(vec![id]);
    }

    Some(())
}

async fn handle_click(mouse_event: MouseEvent, app: &App) -> Option<()> {
    let pos = get_pos(mouse_event);
    let mut tree_state = &mut app.get_focused_tree_state_mut()?;
    let id = tree_state
        .rendered_at(pos)
        .map(|vec| *vec.last().unwrap())?;
    activate_menu_item(id, app, &mut tree_state).await?;
    None
}

fn handle_scroll(mouse_event: MouseEvent, app: &App) -> Option<()> {
    let mut tree_state = app.get_focused_tree_state_mut()?;
    match mouse_event.kind {
        MouseEventKind::ScrollUp => {
            tree_state.scroll_up(1);
        }
        MouseEventKind::ScrollDown => {
            tree_state.scroll_down(1);
        }
        _ => {}
    }
    None
}

async fn handle_move(mouse_event: MouseEvent, app: &mut App) -> Option<()> {
    let pos = get_pos(mouse_event);
    if let Some(index) = app.focused_sni {
        if let Some((_, sni_state)) = app.sni_states.get_index_mut(index) {
            if sni_state.rect.contains(pos) {
                let mut tree_state = app.get_focused_tree_state_mut()?;
                let rendered = tree_state.rendered_at(pos)?.to_owned();
                tree_state.select(rendered.to_vec());
                return None;
            } else {
                sni_state.set_focused(false);
                app.focused_sni = None;
            }
        }
    }

    if let Some(k) = &app.get_focused_sni_key_by_position(pos) {
        if let Some(state_tree) = app.sni_states.get_mut(k) {
            state_tree.set_focused(true);
            app.focused_sni = app.sni_states.get_index_of(k);
        }
    } else {
        app.focused_sni = None;
    }
    Some(())
}

pub async fn handle_mouse_event(mouse_event: MouseEvent, app: &mut App) -> AppResult<()> {
    match mouse_event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let _ = handle_click(mouse_event, app).await;
        }
        MouseEventKind::Down(MouseButton::Right) => {}
        MouseEventKind::Down(MouseButton::Middle) => {}
        MouseEventKind::Moved => {
            let _ = handle_move(mouse_event, app).await;
        }
        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
            let _ = handle_scroll(mouse_event, app);
        }
        _ => {}
    }
    Ok(())
}
