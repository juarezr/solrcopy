#![allow(dead_code)]

// region ProgressBar

use crossbeam_channel::Receiver;
use indicatif::{ProgressBar, ProgressBarIter, ProgressIterator, ProgressStyle};

use crate::helpers::*;

fn new_style(style_template: &str) -> ProgressStyle {
    ProgressStyle::default_bar().template(style_template)
}

pub fn new_wide_style() -> ProgressStyle {
    new_style("[{elapsed_precise} | {eta_precise} | {pos}/{len} | {percent}%] [{wide_bar}]")
}

pub fn new_bar(len: u64) -> ProgressBar {
    ProgressBar::new(len)
}

pub fn new_wide_bar(len: u64) -> ProgressBar {
    ProgressBar::new(len).with_style(new_wide_style())
}

pub fn get_wide_bar_for<S, It: Iterator<Item = S>>(steps: It, len: u64) -> ProgressBarIter<It> {
    steps.progress_with(new_wide_bar(len))
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

// endregion
