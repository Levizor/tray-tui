use std::{cell::RefCell, ops::Deref};

use ratatui::{
    buffer::Buffer,
    layout::{self, Rect},
    style::{Color, Style},
    text::ToLine,
    widgets::Widget,
};
use system_tray::{item::StatusNotifierItem, menu::TrayMenu};

#[derive(Debug)]
pub struct KeyRect {
    pub key: String,
    pub rect: Rect,
    pub focused: bool,
}

impl KeyRect {
    pub fn new(key: String) -> Self {
        Self {
            key,
            rect: Rect::default(),
            focused: false,
        }
    }

    pub fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl Ord for KeyRect {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl PartialOrd for KeyRect {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl PartialEq for KeyRect {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}

impl Eq for KeyRect {}

/// Wrapper around set of StatusNotifierItem and TrayMenu
#[derive(Debug)]
pub struct Item<'a> {
    pub item: StatusNotifierItemW<'a>,
    pub menu: Option<TrayMenuW<'a>>,
    pub rect: Rect,
    pub is_focused: bool,
}

impl<'a> Item<'a> {
    pub fn new((item, menu): &'a (StatusNotifierItem, Option<TrayMenu>)) -> Self {
        Self {
            item: StatusNotifierItemW::new(item),
            menu: menu.as_ref().map(TrayMenuW::new),
            rect: Rect::default(),
            is_focused: false,
        }
    }

    pub fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }
}

impl Widget for Item<'_> {
    fn render(self, area: layout::Rect, buf: &mut Buffer) {
        if self.is_focused {
            if let Some(menu) = self.menu {
                menu.render(area, buf);
                return;
            }
        }

        self.item.render(area, buf);
    }
}

#[derive(Debug)]
pub struct StatusNotifierItemW<'a> {
    inner: &'a StatusNotifierItem,
}

impl<'a> StatusNotifierItemW<'a> {
    pub fn new(sni: &'a StatusNotifierItem) -> StatusNotifierItemW<'a> {
        Self { inner: sni }
    }

    pub fn get_title(&self) -> &String {
        if let Some(title) = &self.title {
            if !title.is_empty() {
                return &title;
            }
        }

        if let Some(tooltip) = &self.tool_tip {
            return &tooltip.title;
        }

        &self.id
    }
}

impl Deref for StatusNotifierItemW<'_> {
    type Target = StatusNotifierItem;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Widget for StatusNotifierItemW<'_> {
    fn render(self, area: layout::Rect, buf: &mut Buffer) {
        let line = self.get_title().to_line();
        log::info!("Rendering widget for line {}", line);

        buf.set_style(area, Style::default().fg(Color::White).bg(Color::Black));
        buf.set_line(
            area.x + (area.width.saturating_sub(line.width() as u16)) / 2,
            area.y + (area.height.saturating_sub(1)) / 2,
            &line,
            area.width,
        );
    }
}

#[derive(Debug)]
pub struct TrayMenuW<'a> {
    inner: &'a TrayMenu,
}

impl<'a> TrayMenuW<'a> {
    pub fn new(menu: &'a TrayMenu) -> Self {
        Self { inner: menu }
    }
}

impl Deref for TrayMenuW<'_> {
    type Target = TrayMenu;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Widget for TrayMenuW<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
    }
}
