use std::{error::Error, fs::File, io};

use crate::{
    app::{App, AppResult},
    config::Config,
    event::{Event, EventHandler},
    handler::handle_key_events,
    tui::Tui,
};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use ratatui::{backend::CrosstermBackend, Terminal};
use simplelog::{CombinedLogger, Config as Conf, LevelFilter, WriteLogger};

use system_tray::{client::Client, error};

pub mod app;
pub mod config;
pub mod event;
pub mod handler;
pub mod tui;
pub mod ui;

#[tokio::main]
async fn main() -> AppResult<()> {
    let config = Config::parse();

    if let Some(shell) = config.completions {
        let mut cmd = Config::command();
        let mut out = io::stdout();
        generate(shell, &mut cmd, "tray-tui", &mut out);
        return Ok(());
    }

    if config.debug {
        CombinedLogger::init(vec![WriteLogger::new(
            LevelFilter::Debug,
            Conf::default(),
            File::create("app.log").unwrap(),
        )])
        .unwrap();
    }

    let client = Client::new().await.unwrap();
    log::info!("Client is initialized");
    let mut tray_rx = client.subscribe();

    // Create an application.
    let mut app = App::new(client);

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;
    log::info!("Initialized TUI");

    while app.running {
        tui.draw(&mut app)?;

        tokio::select! {
            Ok(update) = tray_rx.recv() => {
                log::info!("UPDATE: {:?}", update);
                if let Err(error) = app.feed(update) {
                    log::error!("{:?}", error);
                }
            }

            Ok(event) = tui.events.next() => {
                match event {
                    Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
                    Event::Mouse(mouse_event) => {
                        //handle_mouse_event(mouse_event, &mut app)?
                    },
                    Event::Resize(_, _) => {tui.draw(&mut app).unwrap()}
                    Event::Tick => {}

                }
            }
        };
    }

    log::info!("Exiting application");
    tui.exit()?;
    Ok(())
}
