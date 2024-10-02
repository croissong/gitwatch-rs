use std::thread;

use anyhow::{bail, Result};
use gitwatch_rs::app::App;

pub struct AppRunner {
    handle: thread::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
    shutdown_tx: std::sync::mpsc::Sender<()>,
}

impl AppRunner {
    pub fn run(app: App) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let handle = thread::spawn(move || {
            app.run(Some(rx))?;
            Ok(())
        });

        Self {
            handle,
            shutdown_tx: tx,
        }
    }

    pub fn shutdown(self) -> Result<()> {
        self.shutdown_tx.send(()).unwrap_or_default();
        if let Err(e) = self.handle.join().expect("Thread panicked") {
            bail!(e);
        }
        Ok(())
    }
}
