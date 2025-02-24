use std::cell::RefCell;

use ratatui::widgets::{Block, StatefulWidget};
use ratatui::{
    buffer::Buffer,
    layout::{self, Rect},
    style::{Color, Style},
    widgets::Widget,
};
use system_tray::{
    item::StatusNotifierItem,
    menu::{MenuItem, TrayMenu},
};

use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::config::Config;

#[derive(Debug)]
pub struct SniState {
    pub rect: Rect,
    pub focused: bool,
    pub tree_state: RefCell<TreeState<i32>>,
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

    pub fn get_title(&self) -> &String {
        if let Some(title) = &self.item.title {
            if !title.is_empty() {
                return &title;
            }
        }

        if let Some(tooltip) = &self.item.tool_tip {
            return &tooltip.title;
        }

        &self.item.id
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

    pub fn get_border_color(&self) -> Color {
        let colors = &self.config.colors;
        match self.sni_state.focused {
            true => colors.border_focused,
            false => colors.border,
        }
    }
}

impl Widget for Item<'_> {
    fn render(self, area: layout::Rect, buf: &mut Buffer) {
        let title = self.get_title().clone();
        let (bg, fg) = self.get_colors();
        let (bg_h, fg_h) = self.get_highlight_colors();
        let border = self.get_border_color();

        if let Some(menu) = self.menu {
            let children = menuitems_to_treeitems(&menu.submenus);

            let tree = Tree::new(&children).ok();

            if let Some(mut tree) = tree {
                tree = tree
                    .style(Style::default().bg(bg).fg(fg))
                    .highlight_style(Style::default().bg(bg_h).fg(fg_h));
                tree = tree.block(
                    Block::bordered()
                        .title(title)
                        .border_style(Style::default().fg(border)),
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
