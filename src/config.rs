use ::config;
use ratatui::style::Color;
use serde::Deserialize;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;

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

    #[serde(default = "colors")]
    pub colors: Colors,
}

impl Default for Config {
    fn default() -> Self {
        Self {
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

#[derive(Deserialize, Debug, Clone)]
pub struct Colors {
    #[serde(default = "reset")]
    pub bg: Color,

    #[serde(default = "white")]
    pub fg: Color,

    #[serde(default = "white")]
    pub border: Color,

    #[serde(default = "reset")]
    pub bg_focused: Color,

    #[serde(default = "white")]
    pub fg_focused: Color,

    #[serde(default = "green")]
    pub border_focused: Color,

    #[serde(default = "green")]
    bg_highlighted: Color,

    #[serde(default = "black")]
    fg_highlighted: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            bg: reset(),
            fg: white(),
            border: white(),
            bg_focused: reset(),
            fg_focused: white(),
            border_focused: green(),
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

const fn darkgray() -> Color {
    Color::DarkGray
}

const fn blue() -> Color {
    Color::Blue
}

const fn green() -> Color {
    Color::Green
}
