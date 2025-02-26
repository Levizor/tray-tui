use ratatui::style::Color;
use serde::Deserialize;
use std::error::Error;
use std::path::PathBuf;

fn get_default_config_path() -> Result<PathBuf, Box<dyn Error>> {
    match dirs::config_dir() {
        Some(conf_dir) => Ok(conf_dir.join("tray-tui/config.toml")),
        None => Err(Box::<dyn Error>::from(
            "Couldn't determine default config directory.",
        )),
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "allignment")]
    pub allignment: Allignment,

    #[serde(default = "sorting")]
    pub sorting: bool,

    #[serde(default = "colors")]
    pub colors: Colors,

    #[serde(default = "symbols")]
    pub symbols: Symbols,
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
                let path = get_default_config_path()?;
                if !path.exists() {
                    log::info!("Config file not found. Using default configuration.");
                    return Ok(Self::default());
                }
                builder.add_source(
                    config::File::from(get_default_config_path().expect("Infallible"))
                        .format(config::FileFormat::Toml),
                )
            }
        };

        let config = builder.build()?.try_deserialize::<Config>()?;
        Ok(config)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Allignment {
    Horizontal,
    Vertical,
}

const fn allignment() -> Allignment {
    Allignment::Horizontal
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
