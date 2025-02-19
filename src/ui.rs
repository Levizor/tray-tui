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
        app.keys.iter().for_each(|k| {
            let item = Item::new(
                k.key.clone(),
                items.get(&k.key).expect("not possible (I guess)"),
                &app,
            );
            items_vec.push(item);
        });

        rectangles = Layout::horizontal(iter::repeat(Constraint::Fill(1)).take(items_vec.len()))
            .split(frame.area());

        render_items(frame, items_vec, rectangles.iter());
    }

    app.keys
        .iter_mut()
        .zip(rectangles.iter())
        .for_each(|(k, ar)| k.set_rect(*ar));

    log::info!("{:?}", app.keys);
}

fn render_items(frame: &mut Frame, items: Vec<Item>, rects_iter: std::slice::Iter<'_, Rect>) {
    items.into_iter().zip(rects_iter).for_each(|(item, ar)| {
        frame.render_widget(item, *ar);
    });
}
