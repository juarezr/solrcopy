use log::{debug, error};
use simplelog::{ColorChoice, CombinedLogger, Config, SharedLogger, TermLogger, WriteLogger};
use std::fs::File;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::args::LoggingArgs;

// region Ctrl + C

pub(crate) fn monitor_term_sinal() -> Arc<AtomicBool> {
    lazy_static! {
        static ref ABORTING: Arc<AtomicBool> = start_monitoring_term_sinal();
    }
    ABORTING.clone()
}

fn start_monitoring_term_sinal() -> Arc<AtomicBool> {
    let handler = Arc::new(AtomicBool::new(false));
    let aborting = handler.clone();

    ctrlc::set_handler(move || {
        if handler.aborted() {
            error!("# Received abort signal (Ctrl-C) from user again!!! Aborting...\n");
            std::process::abort();
        } else {
            error!("# Received Ctrl-C signal!!! Stopping threads...\n");
            handler.store(true, Ordering::SeqCst);
        }
    })
    .expect("Error setting Ctrl-C handler");

    debug!("# Waiting for Ctrl-C...");

    aborting
}

// endregion

// region Logging

impl LoggingArgs {
    pub(crate) fn start_log(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut enabled: Vec<Box<dyn SharedLogger>> = Vec::new();
        if !self.is_quiet() {
            enabled.push(TermLogger::new(
                self.log_level,
                Config::default(),
                self.log_mode,
                ColorChoice::Auto,
            ));
        }
        if let Some(filepath) = &self.log_file_path {
            let file_to_log = File::create(filepath).unwrap();
            enabled.push(WriteLogger::new(self.log_level, Config::default(), file_to_log));
        }
        CombinedLogger::init(enabled).unwrap();
        Ok(())
    }
}

// endregion

// region trait Suggaring

pub(crate) trait UserInterruption // where
// Self: Sized,
{
    fn aborted(&self) -> bool;
}

impl UserInterruption for Arc<AtomicBool> {
    fn aborted(&self) -> bool {
        self.load(Ordering::SeqCst)
    }
}

// endregion
