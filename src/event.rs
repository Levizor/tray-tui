use std::collections::HashMap;

use crokey::KeyCombination;
use crossterm::event::{Event as CrosstermEvent, MouseEvent, KeyEventKind};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

use crate::app::AppResult;
use crate::config::KeyBindEvent;

/// Terminal events.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Key press.
    Key(KeyBindEvent),
    /// Mouse click/scroll.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
    /// Loosing focus, doesnt' happen however :(    
    FocusLost,
}

/// Terminal event handler.
#[allow(dead_code)]
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Event>,
    /// Event receiver channel.
    receiver: mpsc::UnboundedReceiver<Event>,
    /// Event handler thread.
    handler: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new(use_mouse: bool, keymap: HashMap<KeyCombination, KeyBindEvent>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let _sender = sender.clone();
        let handler = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            loop {
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  _ = _sender.closed() => {
                    break;
                  }
                  Some(Ok(evt)) = crossterm_event => {
                    match evt {
                      CrosstermEvent::Key(key) => {
                        if key.kind == KeyEventKind::Press {
                          let key_bind = KeyCombination::from(key);
                          if let Some(event) = keymap.get(&key_bind) {
                            _sender.send(Event::Key(*event)).unwrap();
                          }
                        }
                      },
                      CrosstermEvent::Mouse(mouse) => {
                        if use_mouse {
                            _sender.send(Event::Mouse(mouse)).unwrap();
                        }
                      },
                      CrosstermEvent::Resize(x, y) => {
                        _sender.send(Event::Resize(x, y)).unwrap();
                      },
                      CrosstermEvent::FocusLost => {
                        _sender.send(Event::FocusLost).unwrap();
                      },
                      CrosstermEvent::FocusGained => {
                      },
                      CrosstermEvent::Paste(_) => {
                      },
                    }
                  }
                };
            }
        });
        Self {
            sender,
            receiver,
            handler,
        }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub async fn next(&mut self) -> AppResult<Event> {
        self.receiver
            .recv()
            .await
            .ok_or(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "This is an IO error",
            )))
    }
}
