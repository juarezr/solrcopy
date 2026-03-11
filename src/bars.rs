#![allow(dead_code)]

use crossbeam_channel::Receiver;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};

// region ProgressBar

fn new_style(style_template: &str) -> ProgressStyle {
    ProgressStyle::default_bar().template(style_template).unwrap()
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

pub(crate) fn foreach_progress(reporter: Receiver<u64>, total: u64, quiet: bool) -> u64 {
    let mut updated = 0;
    let perc_bar = if quiet { None } else { Some(new_wide_bar(total)) };
    for num_increment in reporter.iter() {
        if let Some(prog) = &perc_bar {
            prog.inc(num_increment);
            updated += num_increment;
        }
    }
    if let Some(pg) = perc_bar {
        pg.finish_and_clear();
    }
    drop(reporter);
    updated
}

pub(crate) fn forall_progress(reporter: Receiver<u64>, total: u64, quiet: bool) -> u64 {
    let mut updated = 0;
    let perc_bar = if quiet { None } else { Some(new_wide_bar(total)) };
    for position in reporter.iter() {
        if position > updated
            && let Some(prog) = &perc_bar
        {
            prog.set_position(position);
            updated = position;
        }
    }
    if let Some(pg) = perc_bar {
        pg.finish_and_clear();
    }
    drop(reporter);
    updated
}

pub(crate) fn wait_with_progress(millis: u64, message: &str) {
    if millis > 10 {
        let delta = millis.min(500);
        let delay = Duration::from_millis(delta);

        let started = Instant::now();
        let deadline = started + Duration::from_millis(millis);

        let time_bar = new_time_bar(millis);
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
