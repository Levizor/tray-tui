use std::{fs::File, io};

use crate::{
    app::{App, AppResult},
    cli::Cli,
    config::Config,
    event::{Event, EventHandler},
    handler::{handle_key_events, handle_mouse_event},
    tui::Tui,
    wrappers::LoggableEvent,
};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use ratatui::{backend::CrosstermBackend, Terminal};
use simplelog::{CombinedLogger, Config as Conf, LevelFilter, WriteLogger};

use system_tray::{client::Client, item::StatusNotifierItem, menu::TrayMenu};

pub mod app;
pub mod cli;
pub mod config;
pub mod event;
pub mod handler;
pub mod tui;
pub mod ui;
pub mod wrappers;

static CMD: &str = "tray-tui";

#[tokio::main]
async fn main() -> AppResult<()> {
    let cli = Cli::parse();

    if let Some(shell) = cli.completions {
        let mut cmd = Cli::command();
        let mut out = io::stdout();
        generate(shell, &mut cmd, CMD, &mut out);
        return Ok(());
    }

    if cli.debug {
        CombinedLogger::init(vec![WriteLogger::new(
            LevelFilter::Debug,
            Conf::default(),
            File::create("app.log").unwrap(),
        )])
        .unwrap();
    }

    let config = Config::new(&cli.config_path)?;

    let client = Client::new().await.unwrap();
    log::info!("Client is initialized");
    let mut tray_rx = client.subscribe();

    log::info!(
        "status: {}, traymenu {}, client {}",
        size_of::<StatusNotifierItem>(),
        size_of::<TrayMenu>(),
        size_of_val(&client)
    );

    // Create an application.
    let mut app = App::new(client, config);
    let map = app.config.key_map.clone();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(app.config.mouse, map);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;
    log::info!("Initialized TUI");

    tui.draw(&mut app)?;

    while app.running {
        tui.draw(&mut app)?;
        tokio::select! {
            Ok(update) = tray_rx.recv() => {
                log::debug!("{}", LoggableEvent(&update));
                app.update();
                if let system_tray::client::Event::Remove(_) = update {
                    app.sync_focus();
                }
            }

            Ok(event) = tui.events.next() => {
                log::debug!("Key event: {:?}", &event);
                match event {
                    Event::Key(key_event) => handle_key_events(key_event, &mut app).await?,
                    Event::Mouse(mouse_event) => {
                        handle_mouse_event(mouse_event, &mut app).await?
                    },
                    Event::Resize(_, _) => {tui.draw(&mut app).unwrap()}
                    Event::FocusLost => {
                        // doensn't work for some reason
                    }
                }
            }
        };
    }

    log::info!("Exiting application");
    tui.exit()?;
    Ok(())
}
