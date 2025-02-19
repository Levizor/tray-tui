use std::rc::Rc;

use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::ToLine,
    Frame,
};

use crate::app::App;
use crate::wrappers::Item;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let mut rectangles: Rc<[Rect]> = Rc::default();
    if let Some(items) = app.get_items() {
        let mut items_vec: Vec<Item> = Vec::new();
        app.keys.iter().for_each(|k| {
            let mut item = Item::new(items.get(&k.key).expect("not possible (I guess)"), &app);
            item.set_focused(k.focused);
            items_vec.push(item);
        });

        rectangles = Layout::horizontal(items_vec.iter().map(|item| {
            let length = (item.item.get_title().to_line().width()
                + frame.area().width as usize / items_vec.len()) as u16;
            Constraint::Length(length)
        }))
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
