use log::{debug, error};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// region Ctrl + C

pub fn monitor_term_sinal() -> Arc<AtomicBool> {
    lazy_static! {
        static ref ABORTING: Arc<AtomicBool> = start_monitoring_term_sinal();
    }
    ABORTING.clone()
}

fn start_monitoring_term_sinal() -> Arc<AtomicBool> {
    let handler = Arc::new(AtomicBool::new(false));
    let aborting = handler.clone();

    ctrlc::set_handler(move || {
        error!("# Received Ctrl-C signal!!! Aborting...\n");

        handler.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    debug!("# Waiting for Ctrl-C...");

    aborting
}

// endregion

// region trait Suggaring

pub trait UserInterruption
where
    Self: Sized,
{
    fn aborted(&self) -> bool;
}

impl UserInterruption for Arc<AtomicBool> {
    fn aborted(&self) -> bool {
        self.load(Ordering::SeqCst)
    }
}

// endregion
