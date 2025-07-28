use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

use anyhow::{Context, Result};
use log::{debug, error, info, trace, warn};
use notify_debouncer_full::{
    new_debouncer,
    notify::{EventKind, RecursiveMode},
    DebouncedEvent,
};

pub struct FileWatcher {
    debounce_seconds: u64,
    retry_count: i32,
}

impl FileWatcher {
    pub fn new(debounce_seconds: u64, retry_count: i32) -> Self {
        Self {
            debounce_seconds,
            retry_count,
        }
    }

    pub fn watch<F, P>(
        &self,
        path: &Path,
        on_change: F,
        is_path_ignored: P,
        shutdown_rx: Option<Receiver<()>>,
    ) -> Result<()>
    where
        F: Fn(&Vec<PathBuf>) -> Result<()>,
        P: Fn(&Path) -> bool,
    {
        let (tx, rx) = mpsc::channel();

        let mut debouncer = new_debouncer(Duration::from_secs(self.debounce_seconds), None, tx)?;

        debouncer
            .watch(path, RecursiveMode::Recursive)
            .context("Failed to watch path")?;
        info!("Watching for changes...");

        loop {
            if let Some(rx) = &shutdown_rx {
                if rx.try_recv().is_ok() {
                    debug!("Received shutdown signal");
                    break;
                }
            }

            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(received) => match received {
                    Ok(events) => {
                        if let Err(e) = self.handle_events(events, &on_change, &is_path_ignored) {
                            error!("All retry attempts failed: {e}");
                            return Err(e);
                        }
                    }
                    Err(errors) => errors.iter().for_each(|error| error!("{error:?}")),
                },
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        Ok(())
    }

    fn handle_events<F, P>(
        &self,
        events: Vec<DebouncedEvent>,
        on_change: F,
        is_path_ignored: P,
    ) -> Result<()>
    where
        F: Fn(&Vec<PathBuf>) -> Result<()>,
        P: Fn(&Path) -> bool,
    {
        trace!("Received notify events {{ events {events:?} }}");
        let paths: Vec<_> = events
            .iter()
            .filter(|event| !matches!(event.kind, EventKind::Access(_) | EventKind::Other))
            .flat_map(|event| event.paths.clone())
            .filter(|path| !is_path_ignored(path))
            .collect();

        if !paths.is_empty() {
            let mut retry_count = 0;
            loop {
                match on_change(&paths) {
                    Ok(()) => break,
                    Err(e) => {
                        if retry_count == self.retry_count {
                            return Err(e);
                        }
                        retry_count += 1;
                        warn!(
                            "Failed to commit changes. Retrying... ({}/{}).\nError: {:?}",
                            retry_count, self.retry_count, e
                        );
                        thread::sleep(RETRY_DELAY);
                    }
                }
            }
        }
        Ok(())
    }
}

const RETRY_DELAY: Duration = Duration::from_secs(1);

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::{
            atomic::{AtomicBool, AtomicU32, Ordering},
            mpsc, Arc,
        },
        thread,
        time::{Duration, Instant},
    };

    use anyhow::bail;
    use notify_debouncer_full::notify::Event;
    use testresult::TestResult;

    use super::*;

    #[test]
    fn test_watcher_notify_error() -> TestResult {
        let dir = tempfile::tempdir()?;
        let path = dir.path().to_path_buf();

        let watcher = FileWatcher::new(0, 2);

        // delete the directory being watched
        fs::remove_dir_all(&path)?;

        // try to watch - should get notify error
        let result = watcher.watch(&path, |_| Ok(()), |_path| false, None);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();

        assert!(
            err.contains("Failed to watch path"),
            "Unexpected error message: {err}"
        );

        Ok(())
    }

    #[test]
    fn test_watcher_callback_error() -> TestResult {
        let temp_dir = tempfile::tempdir()?;
        let test_file = temp_dir.path().join("test.txt");

        // track number of retries
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        let watcher = FileWatcher::new(0, 2);
        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        // start watching in a separate thread
        let temp_dir_path = temp_dir.path().to_owned();
        let handle = thread::spawn(move || {
            watcher.watch(
                &temp_dir_path,
                |_| {
                    attempt_count_clone.fetch_add(1, Ordering::SeqCst);
                    bail!("Mock callback error")
                },
                |_path| false,
                Some(shutdown_rx),
            )
        });

        // give the watcher time to initialize
        thread::sleep(Duration::from_millis(100));

        fs::write(&test_file, "initial content")?;

        // Wait for all retries (2 retries * 1 second sleep between retries)
        thread::sleep(Duration::from_secs(2));

        let _ = shutdown_tx.send(());

        match handle.join().expect("Thread panicked") {
            Ok(_) => panic!("Expected an error from watcher"),
            Err(e) => {
                assert!(e.to_string().contains("Mock callback error"));
                // Initial attempt + 2 retries = 3 total attempts
                assert_eq!(
                    attempt_count.load(Ordering::SeqCst),
                    3,
                    "Expected 3 attempts (1 initial + 2 retries)"
                );
            }
        }

        Ok(())
    }

    #[test]
    fn test_watcher_shutdown() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let test_file = temp_dir.path().join("test.txt");
        let test_file_2 = temp_dir.path().join("test2.txt");
        let temp_dir_path = temp_dir.path().to_owned();

        // create channels for shutdown signal
        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        let watcher = FileWatcher::new(1, 0);

        // Create a counter to track number of changes
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        // Spawn watcher in separate thread
        let handle = thread::spawn(move || {
            watcher.watch(
                &temp_dir_path,
                |_| {
                    counter_clone.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                },
                |_| false,
                Some(shutdown_rx),
            )
        });

        // sleep briefly to ensure watcher is running
        thread::sleep(Duration::from_millis(100));

        fs::write(test_file, "test content")?;

        // wait for debounce
        thread::sleep(Duration::from_secs(3));

        // verify that changes were detected before shutdown
        let counter_before_shutdown = counter.load(Ordering::SeqCst);
        assert!(counter_before_shutdown > 0);

        shutdown_tx.send(())?;
        handle.join().unwrap()?;

        // create another file after shutdown
        fs::write(test_file_2, "test content")?;
        thread::sleep(Duration::from_secs(2));

        // verify no additional changes were detected
        assert_eq!(counter_before_shutdown, counter.load(Ordering::SeqCst));

        Ok(())
    }

    #[test]
    fn test_all_paths_ignored() -> Result<()> {
        let was_called = AtomicBool::new(false);
        let watcher = FileWatcher::new(0, 0);

        let events = vec![
            DebouncedEvent::new(
                Event::new(EventKind::Any).add_path(PathBuf::from("test1.txt")),
                Instant::now(),
            ),
            DebouncedEvent::new(
                Event::new(EventKind::Any).add_path(PathBuf::from("test1.txt")),
                Instant::now(),
            ),
            DebouncedEvent::new(
                Event::new(EventKind::Any).add_path(PathBuf::from("test1.txt")),
                Instant::now(),
            ),
        ];

        watcher.handle_events(
            events,
            |_| {
                was_called.store(true, Ordering::SeqCst);
                Ok(())
            },
            |_| true,
        )?;

        assert!(!was_called.load(Ordering::SeqCst));

        Ok(())
    }
}
