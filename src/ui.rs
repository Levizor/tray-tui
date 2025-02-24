use std::rc::Rc;

use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};
use std::iter;

use crate::app::App;
use crate::wrappers::Item;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let mut rectangles: Rc<[Rect]> = Rc::default();
    if let Some(items) = app.get_items() {
        let mut items_vec: Vec<Item> = Vec::new();
        app.sni_states.iter().for_each(|(k, v)| {
            let item = Item::new(v, items.get(k).expect("not possible (I hope)"), &app.config);
            items_vec.push(item);
        });

        rectangles = Layout::horizontal(iter::repeat(Constraint::Fill(1)).take(items_vec.len()))
            .split(frame.area());

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
