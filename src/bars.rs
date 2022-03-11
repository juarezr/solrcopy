#![allow(dead_code)]

// region ProgressBar

use crossbeam_channel::Receiver;
use indicatif::{ProgressBar, ProgressStyle};

use std::time::{Duration, Instant};

use crate::helpers::*;

fn new_style(style_template: &str) -> ProgressStyle {
    ProgressStyle::default_bar().template(style_template)
}

fn new_wide_style() -> ProgressStyle {
    new_style(" [{elapsed_precise} | {eta_precise} | {pos}/{len} | {percent}%] [{wide_bar}] ")
}

fn new_time_style() -> ProgressStyle {
    new_style(" [{elapsed_precise} | {eta_precise} | {percent}%] [{wide_bar}] {msg}")
}

fn new_wide_bar(len: u64) -> ProgressBar {
    ProgressBar::new(len).with_style(new_wide_style())
}

fn new_time_bar(len: u64) -> ProgressBar {
    ProgressBar::new(len).with_style(new_time_style())
}

// endregion

// region implementarion

pub fn foreach_progress(
    reporter: Receiver<u64>, num_retrieving: usize, num_increment: usize, quiet: bool,
) -> usize {
    let mut updated = 0;
    let perc_bar = if quiet { None } else { Some(new_wide_bar(num_retrieving.to_u64())) };
    for _ in reporter.iter() {
        if let Some(prog) = &perc_bar {
            prog.inc(num_increment.to_u64());
            updated += num_increment;
        }
    }
    if let Some(pg) = perc_bar {
        pg.finish_and_clear();
    }
    drop(reporter);
    updated
}

pub fn wait_with_progress(millis: usize, message: &str) {
    if millis > 10 {
        let delta = millis.min(500).to_u64();
        let delay = Duration::from_millis(delta);

        let started = Instant::now();
        let deadline = started + Duration::from_millis(millis.to_u64());

        let time_bar = new_time_bar(millis.to_u64());
        if !message.is_empty() {
            // time_bar.println(message);
            time_bar.set_message(message.to_owned());
        }
        loop {
            let now = Instant::now();
            if now > deadline {
                break;
            }
            time_bar.inc(delta);
            std::thread::sleep(delay);
        }
        time_bar.finish_and_clear();
    }
}

// endregion
