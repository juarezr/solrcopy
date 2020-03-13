#![allow(dead_code)]

// region ProgressBar

use indicatif::{ProgressBar, ProgressBarIter, ProgressIterator, ProgressStyle};

fn new_style(style_template: &str) -> ProgressStyle {
    ProgressStyle::default_bar().template(style_template)
}

pub fn new_wide_style() -> ProgressStyle {
    new_style(
        "{spinner:.green} [{elapsed_precise}] [{wide_bar:40.cyan/blue}] {pos}/{len}  {percent}% \
         ({eta})",
    )
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

pub fn wait(secs: u64) {
    let millis = secs * 1000;
    std::thread::sleep(std::time::Duration::from_millis(millis));
}

// endregion
