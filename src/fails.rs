#![allow(dead_code)]

use std::{error::Error, fmt};

pub type BoxedResult<T> = Result<T, Box<dyn Error>>;

pub type BoxedError = Result<(), Box<dyn Error>>;

pub struct Failed {
    details: String,
}

impl Failed {
    pub fn new(msg: &str) -> Self {
        Self { details: msg.to_string() }
    }

    pub fn from(msg: String) -> Self {
        Self { details: msg }
    }

    fn say(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl fmt::Display for Failed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.say(f)
    }
}

impl fmt::Debug for Failed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.say(f)
    }
}

impl Error for Failed {
    fn description(&self) -> &str {
        &self.details
    }
}

// region utilities

pub fn throw<T>(message: String) -> BoxedResult<T> {
    Err(Box::new(Failed::new(&message)))
}

pub fn failed(message: String) -> impl FnOnce() -> Box<dyn Error> {
    let res: Box<dyn Error> = Box::new(Failed::new(&message));
    move || res
}

pub fn rethrow<T, E>(failure: E) -> BoxedResult<T>
where
    E: Error + 'static,
{
    Err(Box::new(failure))
}

pub fn raise<T>(message: &str) -> BoxedResult<T> {
    Err(Box::new(Failed::new(message)))
}

macro_rules! throws {
    ($($arg:tt)*) => {
        Err(Box::new(crate::fails::Failed::from( format!($($arg)*) )))
    };
}

macro_rules! should_fail {
    ($($arg:tt)*) => {{
        let message = format!($($arg)*);
        let res: Box<dyn std::error::Error> = Box::new(crate::fails::Failed::new(&message));
        move || res
    }};
}

// endregion

#[cfg(test)]
mod tests {
    use crate::fails::{raise, throw};
    use pretty_assertions::assert_eq;

    #[test]
    fn check_throw_and_raise() {
        let failure = throw::<usize>("fail".to_string());
        assert_eq!(failure.is_ok(), false);

        // TODO revamp error handling results types
        // let failerr = failure.err().unwrap();
        // assert_eq!(rethrow(failerr).is_ok(), false);

        assert_eq!(raise::<usize>("fail").is_ok(), false);
    }
}
