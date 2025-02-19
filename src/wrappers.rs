use std::iter;
use std::ops::Deref;

use ratatui::{
    buffer::Buffer,
    layout::{self, Constraint, Layout, Rect},
    style::{Color, Style},
    text::ToLine,
    widgets::Widget,
};
use system_tray::{
    item::StatusNotifierItem,
    menu::{MenuItem, TrayMenu},
};

use crate::app::App;

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
    pub id: String,
    pub item: StatusNotifierItemW<'a>,
    pub menu: Option<TrayMenuW<'a>>,
    pub rect: Rect,
    app: &'a App,
}

impl<'a> Item<'a> {
    pub fn new(
        id: String,
        (item, menu): &'a (StatusNotifierItem, Option<TrayMenu>),
        app: &'a App,
    ) -> Self {
        Self {
            id,
            item: StatusNotifierItemW::new(item),
            menu: menu.as_ref().map(|tm| TrayMenuW::new(tm, app)),
            rect: Rect::default(),
            app,
        }
    }

    pub fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }
}

impl Widget for Item<'_> {
    fn render(self, area: layout::Rect, buf: &mut Buffer) {
        if let Some(i) = &self.app.focused_key {
            if self.id.eq(i) {
                if let Some(menu) = self.menu {
                    menu.render(area, buf);
                    return;
                }
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
    app: &'a App,
}

impl<'a> TrayMenuW<'a> {
    pub fn new(menu: &'a TrayMenu, app: &'a App) -> Self {
        Self { inner: menu, app }
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
        self.app.box_stack.borrow_mut().clear();
        let menu_items = self.submenus.iter().map(|s| MenuItemW::new(s, self.app));
        let rects = Layout::vertical(iter::repeat(Constraint::Fill(1)).take(self.submenus.len()))
            .split(area);

        menu_items.zip(rects.iter()).for_each(|(m, r)| {
            self.app.box_stack.borrow_mut().push((m.id, *r));
            m.render(*r, buf)
        });
    }
}

#[derive(Debug)]
pub struct MenuItemW<'a> {
    inner: &'a MenuItem,
    app: &'a App,
}

impl<'a> MenuItemW<'a> {
    pub fn new(inner: &'a MenuItem, app: &'a App) -> Self {
        Self { inner, app }
    }
}

impl Deref for MenuItemW<'_> {
    type Target = MenuItem;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Widget for MenuItemW<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let line = match &self.label {
            Some(label) => label.to_line(),
            None => self.id.to_line(),
        };
        buf.set_line(
            area.x + (area.width.saturating_sub(line.width() as u16)) / 2,
            area.y + (area.height.saturating_sub(1)) / 2,
            &line,
            area.width,
        );
        if !self.submenu.is_empty() {
            return; // TODO
        };
    }
}
