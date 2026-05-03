use std::fs::OpenOptions;
use std::io::Write;
use std::sync::mpsc::{self, Sender};
use std::thread;

/// Command for the background logging thread
enum LogCommand {
    Log {
        file_path: String,
        message: String,
    },
    Terminate,
}

/// A non-blocking logger for architectural and execution logs.
/// It uses a background thread to perform disk I/O.
pub struct AsyncArchLogger {
    tx: Sender<LogCommand>,
}

impl AsyncArchLogger {
    /// Creates a new `AsyncArchLogger` and starts the background thread.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            while let Ok(command) = rx.recv() {
                match command {
                    LogCommand::Log { file_path, message } => {
                        if let Ok(mut file) = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&file_path)
                        {
                            let _ = writeln!(file, "{}", message);
                            // We don't necessarily need to sync_data here for lint logs
                            // but we could if durability was critical.
                        }
                    }
                    LogCommand::Terminate => break,
                }
            }
        });

        Self { tx }
    }

    /// Sends a log message to the background thread.
    pub fn log(&self, file_path: &str, message: String) {
        let _ = self.tx.send(LogCommand::Log {
            file_path: file_path.to_string(),
            message,
        });
    }
}

impl Default for AsyncArchLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AsyncArchLogger {
    fn drop(&mut self) {
        let _ = self.tx.send(LogCommand::Terminate);
    }
}
