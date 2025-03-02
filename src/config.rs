use crate::CMD;
use crokey::{key, parse, KeyCombination};
use ratatui::style::Color;
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum KeyBindEvent {
    FocusLeft,
    FocusDown,
    FocusUp,
    FocusRight,
    TreeUp,
    TreeDown,
    Quit,
    Activate,
    None,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "allignment")]
    pub allignment: Allignment,

    #[serde(default = "columns")]
    pub columns: usize,

    #[serde(default = "sorting")]
    pub sorting: bool,

    #[serde(default = "colors")]
    pub colors: Colors,

    #[serde(default = "symbols")]
    pub symbols: Symbols,

    #[serde(default = "mouse")]
    pub mouse: bool,

    #[serde(default = "key_map", deserialize_with = "merge_with_default")]
    pub key_map: HashMap<KeyCombination, KeyBindEvent>,
}

fn merge_with_default<'de, D>(
    deserializer: D,
) -> Result<HashMap<KeyCombination, KeyBindEvent>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut default_map = key_map();
    let config_map: Option<HashMap<KeyCombination, KeyBindEvent>> =
        Option::deserialize(deserializer)?;
    if let Some(map) = config_map {
        default_map.extend(map);
    }

    Ok(default_map)
}

#[derive(Deserialize, Debug)]
pub struct Symbols {
    #[serde(default = "highlight_symbol")]
    pub highlight_symbol: String,

    #[serde(default = "node_closed_symbol")]
    pub node_closed_symbol: String,

    #[serde(default = "node_open_symbol")]
    pub node_open_symbol: String,

    #[serde(default = "node_no_children_symbol")]
    pub node_no_children_symbol: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Colors {
    #[serde(default = "reset")]
    pub bg: Color,

    #[serde(default = "white")]
    pub fg: Color,

    #[serde(default = "white")]
    pub border_fg: Color,

    #[serde(default = "reset")]
    pub border_bg: Color,

    #[serde(default = "reset")]
    pub bg_focused: Color,

    #[serde(default = "white")]
    pub fg_focused: Color,

    #[serde(default = "green")]
    pub border_fg_focused: Color,

    #[serde(default = "reset")]
    pub border_bg_focused: Color,

    #[serde(default = "green")]
    pub bg_highlighted: Color,

    #[serde(default = "black")]
    pub fg_highlighted: Color,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Allignment {
    Horizontal,
    Vertical,
}

impl Default for Symbols {
    fn default() -> Self {
        Self {
            highlight_symbol: highlight_symbol(),
            node_open_symbol: node_open_symbol(),
            node_closed_symbol: node_closed_symbol(),
            node_no_children_symbol: node_no_children_symbol(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sorting: sorting(),
            symbols: symbols(),
            allignment: allignment(),
            colors: colors(),
            columns: columns(),
            mouse: mouse(),
            key_map: key_map(),
        }
    }
}

impl Config {
    pub fn new(path: &Option<PathBuf>) -> Result<Self, Box<dyn Error>> {
        let builder = config::Config::builder();
        let builder = match path {
            Some(path) => builder
                .add_source(config::File::from(path.clone()).format(config::FileFormat::Toml)),
            None => {
                let path = Self::get_default_config_path()?;
                if !path.exists() {
                    log::info!("Config file not found. Using default configuration.");
                    return Ok(Self::default());
                }
                builder.add_source(config::File::from(path).format(config::FileFormat::Toml))
            }
        };

        let config = builder.build()?.try_deserialize::<Config>()?;
        Ok(config)
    }
    fn get_default_config_path() -> Result<PathBuf, Box<dyn Error>> {
        match dirs::config_dir() {
            Some(conf_dir) => Ok(conf_dir.join(format!("{CMD}/config.toml"))),
            None => Err(Box::<dyn Error>::from(
                "Couldn't determine default config directory.",
            )),
        }
    }
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            bg: reset(),
            fg: white(),
            border_fg: white(),
            border_bg: reset(),
            bg_focused: reset(),
            fg_focused: white(),
            border_fg_focused: green(),
            border_bg_focused: reset(),
            bg_highlighted: green(),
            fg_highlighted: black(),
        }
    }
}

fn colors() -> Colors {
    Colors::default()
}

const fn reset() -> Color {
    Color::Reset
}
const fn black() -> Color {
    Color::Black
}

const fn white() -> Color {
    Color::White
}

const fn green() -> Color {
    Color::Green
}

const fn sorting() -> bool {
    false
}

fn symbols() -> Symbols {
    Symbols::default()
}

fn highlight_symbol() -> String {
    String::new()
}

fn node_closed_symbol() -> String {
    String::from(" ⏷ ")
}

fn node_open_symbol() -> String {
    String::from(" ▶ ")
}

fn node_no_children_symbol() -> String {
    String::from(" ")
}

const fn columns() -> usize {
    3
}

const fn mouse() -> bool {
    true
}

const fn allignment() -> Allignment {
    Allignment::Horizontal
}

fn key_map() -> HashMap<KeyCombination, KeyBindEvent> {
    let mut map = HashMap::new();
    map.insert(key!(left), KeyBindEvent::FocusLeft);
    map.insert(key!(right), KeyBindEvent::FocusRight);
    map.insert(key!(down), KeyBindEvent::FocusDown);
    map.insert(key!(up), KeyBindEvent::FocusUp);
    map.insert(key!(s - tab), KeyBindEvent::TreeUp);
    map.insert(key!(tab), KeyBindEvent::TreeDown);
    map.insert(key!(ctrl - c), KeyBindEvent::Quit);
    map.insert(key!(q), KeyBindEvent::Quit);
    map.insert(key!(enter), KeyBindEvent::Activate);

    map
}
