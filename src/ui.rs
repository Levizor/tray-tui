use std::iter::repeat_n;

use ratatui::{
    layout::{Constraint, Layout, Rect},
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::app::App;
use crate::wrappers::Item;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let mut rectangles: Vec<Rect> = Vec::default();

    let rows = app.layout.rows.len();
    if rows == 0 {
        return;
    }
    let min_height = app.config.min_height;
    let mut area = frame.area();

    let total_min_height = rows as u16 * min_height;

    // Split area for scrollbar only if enabled AND needed
    let scrollbar_area = if app.config.scrollbar && total_min_height > area.height {
        let [main, scrollbar] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
        area = main;
        Some(scrollbar)
    } else {
        None
    };

    let viewport_height = area.height;
    let max_scroll = total_min_height.saturating_sub(viewport_height);

    // Calculate scroll offset to keep focused item visible only if focus changed
    if app.focused_sni_index != app.last_focused_sni_index {
        let focused_row = app
            .layout
            .rows
            .iter()
            .position(|row| row.contains(&app.focused_sni_index))
            .unwrap_or(0);
        let row_top = focused_row as u16 * min_height;
        let row_bottom = row_top + min_height;

        if row_top < app.layout.scroll_offset {
            app.layout.scroll_offset = row_top;
        } else if row_bottom > app.layout.scroll_offset + viewport_height {
            app.layout.scroll_offset = row_bottom.saturating_sub(viewport_height);
            // Snap to row boundary when auto-scrolling down
            app.layout.scroll_offset = (app.layout.scroll_offset / min_height) * min_height;
            if app.layout.scroll_offset + viewport_height < row_bottom {
                 app.layout.scroll_offset = app.layout.scroll_offset.saturating_add(min_height).min(max_scroll);
            }
        }
        app.last_focused_sni_index = app.focused_sni_index;
    }

    app.layout.scroll_offset = app.layout.scroll_offset.min(max_scroll);

    // Render scrollbar if needed
    if let Some(sa) = scrollbar_area {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut state = ScrollbarState::new(total_min_height as usize)
            .position(app.layout.scroll_offset as usize)
            .viewport_content_length(viewport_height as usize);
        frame.render_stateful_widget(scrollbar, sa, &mut state);
    }

    if let Some(items) = app.get_items() {
        let mut items_vec: Vec<Item> = Vec::new();
        app.sni_states.iter().for_each(|(k, v)| {
            if let Some(pair) = items.get(k) {
                let item = Item::new(v, pair, &app.config);
                items_vec.push(item);
            }
        });

        rectangles = {
            let mut result = Vec::new();

            // If we are scrolling, we split an area equal to the total minimum height.
            // If not, we split the viewport so they can expand.
            let split_area = if total_min_height > viewport_height {
                Rect::new(area.x, area.y, area.width, total_min_height)
            } else {
                area
            };

            let row_layout =
                Layout::vertical(repeat_n(Constraint::Min(min_height), rows)).split(split_area);

            for r in 0..rows {
                let items_n = app.layout.rows[r].len();

                let col_layout =
                    Layout::horizontal(repeat_n(Constraint::Fill(1), items_n)).split(row_layout[r]);

                // Culling
                for col_rect in col_layout.iter().copied() {
                    let abs_y = col_rect.y;
                    let scroll_y = app.layout.scroll_offset;

                    if abs_y + col_rect.height <= scroll_y || abs_y >= scroll_y + viewport_height {
                        // Fully outside the viewport
                        result.push(Rect::default());
                    } else {
                        // Partially or fully inside
                        let mut r = col_rect;
                        let y_offset = abs_y as i32 - scroll_y as i32;

                        if y_offset < 0 {
                             // Item is partially above the top
                             r.y = area.y;
                             r.height = (abs_y + col_rect.height).saturating_sub(scroll_y);
                        } else {
                             // Item is below or at the top
                             r.y = area.y + y_offset as u16;
                        }
                        result.push(r.intersection(area));
                    }
                }
            }

            result
        };

        render_items(frame, items_vec, rectangles.iter());
    }

    app.sni_states
        .values_mut()
        .zip(rectangles.iter())
        .for_each(|(v, ar)| v.set_rect(*ar));
}

fn render_items(frame: &mut Frame, items: Vec<Item>, rects_iter: std::slice::Iter<'_, Rect>) {
    items.into_iter().zip(rects_iter).for_each(|(item, ar)| {
        frame.render_widget(item, *ar);
    });
}
