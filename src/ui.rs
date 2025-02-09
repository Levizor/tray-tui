use ratatui::{
    buffer::Buffer,
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, BorderType, Paragraph, WidgetRef},
    Frame,
};

use system_tray::item::StatusNotifierItem;

use crate::app::App;

#[derive(Debug)]
pub struct Item(pub StatusNotifierItem);

impl WidgetRef for Item {
    #[allow(clippy::cast_possible_truncation)]
    fn render_ref(&self, area: ratatui::layout::Rect, buf: &mut Buffer) {}
}

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    frame.render_widget(
        Paragraph::new(format!(
            "This is a tui template.\n\
                Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
                Press left and right to increment and decrement the counter respectively.\n\
                Counter: {}",
            app.get_items().len()
        ))
        .block(
            Block::bordered()
                .title("Template")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Cyan).bg(Color::Black))
        .centered(),
        frame.area(),
    )
}
