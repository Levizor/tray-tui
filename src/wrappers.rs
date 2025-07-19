use std::cell::RefCell;

use ratatui::widgets::{Block, StatefulWidget};
use ratatui::{
    buffer::Buffer,
    layout::{self, Rect},
    style::{Color, Style},
    widgets::Widget,
};
use system_tray::client::{Event, UpdateEvent};
use system_tray::{
    item::StatusNotifierItem,
    menu::{MenuItem, TrayMenu},
};

use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::config::Config;

pub type Id = usize;

#[derive(Debug)]
pub struct SniState {
    pub rect: Rect,
    pub focused: bool,
    pub tree_state: RefCell<TreeState<Id>>,
}

impl SniState {
    pub fn new() -> Self {
        Self {
            rect: Rect::default(),
            focused: false,
            tree_state: RefCell::default(),
        }
    }

    pub fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

pub trait GetTitle {
    fn get_title(&self) -> &String;
}

impl GetTitle for StatusNotifierItem {
    fn get_title(&self) -> &String {
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

/// Wrapper around set of [StatusNotifierItem] and [TrayMenu]
#[derive(Debug)]
pub struct Item<'a> {
    pub sni_state: &'a SniState,
    pub item: &'a StatusNotifierItem,
    pub menu: &'a Option<TrayMenu>,
    config: &'a Config,
    pub rect: Rect,
}

impl<'a> Item<'a> {
    pub fn new(
        sni_state: &'a SniState,
        (item, menu): &'a (StatusNotifierItem, Option<TrayMenu>),
        config: &'a Config,
    ) -> Self {
        Self {
            sni_state,
            item,
            menu,
            config,
            rect: Rect::default(),
        }
    }

    pub fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }

    pub fn get_colors(&self) -> (Color, Color) {
        let colors = &self.config.colors;
        let mut bg = colors.bg;
        let mut fg = colors.fg;

        if self.sni_state.focused {
            bg = colors.bg_focused;
            fg = colors.fg_focused;
        }

        (bg, fg)
    }

    pub fn get_highlight_colors(&self) -> (Color, Color) {
        let colors = &self.config.colors;
        (colors.bg_highlighted, colors.fg_highlighted)
    }

    pub fn get_border_color(&self) -> (Color, Color) {
        let colors = &self.config.colors;
        match self.sni_state.focused {
            true => (colors.border_bg_focused, colors.border_fg_focused),
            false => (colors.border_bg, colors.border_fg),
        }
    }
}

impl Widget for Item<'_> {
    fn render(self, area: layout::Rect, buf: &mut Buffer) {
        let title = self.item.get_title().clone();
        let (bg, fg) = self.get_colors();
        let (bg_h, fg_h) = self.get_highlight_colors();
        let (border_bg, border_fg) = self.get_border_color();
        let symbols = &self.config.symbols;

        if let Some(menu) = self.menu {
            let children = menuitems_to_treeitems(&menu.submenus);

            let tree = Tree::new(&children);

            if let Ok(mut tree) = tree {
                tree = tree
                    .style(Style::default().bg(bg).fg(fg))
                    .highlight_style(Style::default().bg(bg_h).fg(fg_h))
                    .highlight_symbol(&symbols.highlight_symbol)
                    .node_open_symbol(&symbols.node_open_symbol)
                    .node_closed_symbol(&symbols.node_closed_symbol)
                    .node_no_children_symbol(&symbols.node_no_children_symbol);
                tree = tree.block(
                    Block::bordered()
                        .title(title)
                        .border_style(Style::default().fg(border_fg).bg(border_bg)),
                );

                StatefulWidget::render(
                    tree,
                    area,
                    buf,
                    &mut self.sni_state.tree_state.borrow_mut(),
                );
            }
        } else {
            let block = Block::default().title(title).style(Style::default());
            block.render(area, buf);
        }
    }
}

fn menuitem_to_treeitem(id: usize, menu_item: &MenuItem) -> Option<TreeItem<Id>> {
    if menu_item.submenu.is_empty() {
        match &menu_item.label {
            Some(label) => return Some(TreeItem::new_leaf(id, label.clone())),
            None => return None,
        }
    }
    let children = menuitems_to_treeitems(&menu_item.submenu);
    let root = TreeItem::new(
        id,
        menu_item.label.clone().unwrap_or(String::from("no_label")),
        children,
    );

    root.ok()
}

fn menuitems_to_treeitems(menu_items: &Vec<MenuItem>) -> Vec<TreeItem<Id>> {
    menu_items
        .iter()
        .enumerate()
        .map(|(index, menu_item)| menuitem_to_treeitem(index, menu_item))
        .filter_map(|x| x)
        .collect()
}

pub trait FindMenuByUsize {
    fn find_menu_by_usize(&self, ids: &[Id]) -> Option<&MenuItem>;
}

impl FindMenuByUsize for TrayMenu {
    fn find_menu_by_usize(&self, ids: &[Id]) -> Option<&MenuItem> {
        if ids.len() == 0 {
            return None;
        }
        let mut result: &MenuItem = self.submenus.get(ids[0])?;
        let mut submenus = &result.submenu;
        for i in ids.iter().skip(1) {
            result = submenus.get(*i)?;
            submenus = &result.submenu;
        }

        Some(result)
    }
}

pub struct LoggableEvent<'a>(pub &'a system_tray::client::Event);

impl std::fmt::Display for LoggableEvent<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Event::Update(dest, update_event) => {
                write!(
                    f,
                    "{} Update Event for {}",
                    update_event_variant(&update_event),
                    dest
                )
            }
            Event::Add(dest, sni) => write!(f, "Add Event for {}: {}", dest, sni.get_title()),
            Event::Remove(dest) => write!(f, "Remove Event for {}", dest),
        }
    }
}

fn update_event_variant(event: &UpdateEvent) -> &'static str {
    match event {
        UpdateEvent::AttentionIcon(_) => "AttentionIcon",
        UpdateEvent::Icon {
            icon_name: _,
            icon_pixmap: _,
        } => "Icon",
        UpdateEvent::OverlayIcon(_) => "OverlayIcon",
        UpdateEvent::Status(_) => "Status",
        UpdateEvent::Title(_) => "Title",
        UpdateEvent::Tooltip(_) => "Tooltip",
        UpdateEvent::Menu(_) => "Menu",
        UpdateEvent::MenuDiff(_) => "MenuDiff",
        UpdateEvent::MenuConnect(_) => "MenuConnect",
    }
}
