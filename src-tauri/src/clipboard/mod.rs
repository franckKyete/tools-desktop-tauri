use std::time::Duration;

use arboard::Clipboard as ArboardClip;
use tokio::{self, time::sleep};

use crate::event::ClipboardEvent;

use super::event::event_manager::EVENT_MANAGER;

pub struct Clipboard {}

impl Clipboard {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self) {
        tokio::task::spawn(async move {
            let mut clipboard = ArboardClip::new().expect("Failed to initialize clipboard");
            let mut last_content = clipboard.get_text().unwrap_or_default();

            let event_manager = EVENT_MANAGER.clone();

            loop {
                match clipboard.get_text() {
                    Ok(current_content) => {
                        if current_content != last_content {
                            event_manager
                                .emit(ClipboardEvent {
                                    text: current_content.clone(),
                                })
                                .await;
                            last_content = current_content;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading clipboard : {:?}", e);
                    }
                }
                sleep(Duration::from_millis(200)).await;
            }
        });
    }
}
