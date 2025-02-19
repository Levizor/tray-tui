use crate::{
    app::{App, AppResult},
    wrappers::KeyRect,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent};
use ratatui::layout::Position;
use system_tray::client::ActivateRequest;

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

async fn handle_click(mouse_event: MouseEvent, app: &App) {
    let pos = get_pos(mouse_event);
    if let Some(s) = get_focused_status_notifier_keyrect(app, pos) {
        let stack = app.box_stack.borrow();
        let item = stack
            .iter()
            .rev()
            .find(|(_, rect)| rect.contains(get_pos(mouse_event)));

        if let Some((item_id, _)) = item {
            if let Some(map) = app.get_items() {
                if let Some((sni, _menu)) = map.get(&s.key) {
                    if let Some(path) = &sni.menu {
                        let _ = app
                            .client
                            .activate(ActivateRequest::MenuItem {
                                address: s.key.clone(),
                                menu_path: path.to_string(),
                                submenu_id: *item_id,
                            })
                            .await;
                    }
                }
            }
        }
    }
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
                app.box_stack.borrow_mut().clear();
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
