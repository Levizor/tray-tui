use std::ops::{Deref, DerefMut};
use std::{iter, vec};

use ratatui::widgets::{Block, StatefulWidget};
use ratatui::{
    buffer::Buffer,
    layout::{self, Constraint, Layout, Rect},
    style::{Color, Style},
    text::ToLine,
    widgets::Widget,
};
use system_tray::{
    item::StatusNotifierItem,
    menu::{MenuItem, MenuType, TrayMenu},
};

use tui_tree_widget::{Tree, TreeItem, TreeState};

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

/// Wrapper around set of [StatusNotifierItem] and [TrayMenu]
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

fn menuitem_to_treeitem(menu_item: &MenuItem) -> Option<TreeItem<i32>> {
    if menu_item.submenu.is_empty() {
        match &menu_item.label {
            Some(label) => return Some(TreeItem::new_leaf(menu_item.id, label.clone())),
            None => return None,
        }
    }
    let children = menuitems_to_treeitems(&menu_item.submenu);
    let root = TreeItem::new(
        menu_item.id,
        menu_item.label.clone().unwrap_or(String::from("no_label")),
        children,
    );

    root.ok()
}

fn menuitems_to_treeitems(menu_items: &Vec<MenuItem>) -> Vec<TreeItem<i32>> {
    menu_items
        .iter()
        .map(|menu_item| menuitem_to_treeitem(menu_item))
        .filter_map(|x| x)
        .collect()
}

impl Widget for TrayMenuW<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let children = menuitems_to_treeitems(&self.submenus);

        let tree = Tree::new(&children).ok();

        if let Some(tree) = tree {
            StatefulWidget::render(
                tree.block(Block::bordered())
                    .highlight_style(Style::default().bg(Color::Green)),
                area,
                buf,
                &mut self.app.menu_tree_state.borrow_mut(),
            );
        }

        //let menu_items: Vec<_> = self
        //    .submenus
        //    .iter()
        //    .filter(|m| m.menu_type != MenuType::Separator)
        //    .map(|s| MenuItemW::new(s, self.app))
        //    .collect();
        //let rects =
        //    Layout::vertical(iter::repeat(Constraint::Fill(1)).take(menu_items.len())).split(area);
        //
        //menu_items.into_iter().zip(rects.iter()).for_each(|(m, r)| {
        //    self.app.box_stack.borrow_mut().push((m.id, *r));
        //    m.render(*r, buf)
        //});
        //self.app.box_stack.borrow_mut().clear();
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
        let mut label: String = match &self.label {
            Some(label) => label.to_string(),
            None => self.id.to_string(),
        };
        if !self.submenu.is_empty() {
            label.push_str(" ⏵");
        }

        let line = label.to_line();

        buf.set_line(
            area.x + (area.width.saturating_sub(line.width() as u16)) / 2,
            area.y + (area.height.saturating_sub(1)) / 2,
            &line,
            area.width,
        );
        if !self.submenu.is_empty() {
            // TODO
        };
    }
}
