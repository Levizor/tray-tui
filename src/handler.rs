use crate::{
    app::{App, AppResult, FocusDirection},
    config::KeyBindEvent,
};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

/// Handles the key events and updates the state of [`App`].
pub async fn handle_key_events(key_bind_event: KeyBindEvent, app: &mut App) -> AppResult<()> {
    match key_bind_event {
        // Exit application on `ESC` or `q`
        KeyBindEvent::Quit => {
            app.quit();
        }
        KeyBindEvent::FocusLeft => {
            app.move_focus(FocusDirection::Left);
        }
        KeyBindEvent::FocusRight => {
            app.move_focus(FocusDirection::Right);
        }
        KeyBindEvent::FocusDown => {
            app.move_focus(FocusDirection::Down);
        }
        KeyBindEvent::FocusUp => {
            app.move_focus(FocusDirection::Up);
        }
        _ => {}
    }
    let tree_state = app.get_focused_tree_state_mut();
    if tree_state.is_none() {
        return Ok(());
    }
    let mut tree_state = &mut tree_state.unwrap();
    match key_bind_event {
        KeyBindEvent::Activate => {
            let ids = tree_state.selected().to_vec();
            let _ = app.activate_menu_item(&ids, &mut tree_state).await;
        }
        KeyBindEvent::MenuDown => {
            if !tree_state.key_down() {
                tree_state.select_first();
            }
        }
        KeyBindEvent::MenuUp => {
            if !tree_state.key_up() {
                tree_state.select_last();
            }
        }
        _ => {}
    }

    Ok(())
}

fn get_pos(mouse_event: MouseEvent) -> Position {
    Position::new(mouse_event.column, mouse_event.row)
}

async fn handle_click(mouse_event: MouseEvent, app: &App) -> Option<()> {
    let pos = get_pos(mouse_event);
    let mut tree_state = &mut app.get_focused_tree_state_mut()?;
    let ids = tree_state.rendered_at(pos)?.to_vec();
    app.activate_menu_item(&ids, &mut tree_state).await?;
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
    if let Some((_, sni_state)) = app.sni_states.get_index_mut(app.focused_sni_index) {
        if sni_state.rect.contains(pos) {
            let mut tree_state = app.get_focused_tree_state_mut()?;
            let rendered = tree_state.rendered_at(pos)?.to_owned();
            tree_state.select(rendered.to_vec());
            return None;
        } else {
            sni_state.set_focused(false);
            app.focused_sni_index = 0;
        }
    }

    if let Some(k) = &app.get_focused_sni_key_by_position(pos) {
        if let Some(state_tree) = app.sni_states.get_mut(k) {
            state_tree.set_focused(true);
            app.focused_sni_index = app.sni_states.get_index_of(k).unwrap_or(0);
        }
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
