use std::iter::repeat_n;

use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::app::App;
use crate::wrappers::Item;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let mut rectangles: Vec<Rect> = Vec::default();
    if let Some(items) = app.get_items() {
        let mut items_vec: Vec<Item> = Vec::new();
        app.sni_states.iter().for_each(|(k, v)| {
            if let Some(pair) = items.get(k) {
                let item = Item::new(v, pair, &app.config);
                items_vec.push(item);
            }
        });

        rectangles = {
            let size = items_vec.len();
            let columns: usize = app.config.columns;
            let rows: usize = (size + columns - 1) / columns;

            let mut result = Vec::new();

            let row_layout =
                Layout::vertical(repeat_n(Constraint::Fill(1), rows)).split(frame.area());

            for r in 0..rows {
                let start = r * columns;
                let end = (start + columns).min(size);
                let items_n = end - start;

                let col_layout =
                    Layout::horizontal(repeat_n(Constraint::Fill(1), items_n)).split(row_layout[r]);

                result.extend_from_slice(&col_layout[..items_n]);
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
