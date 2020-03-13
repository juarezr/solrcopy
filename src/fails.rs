#![allow(dead_code)]

use std::{error::Error, fmt};

pub type BoxedError = Box<dyn Error>;

pub type BoxedResult<T> = Result<T, BoxedError>;

pub type BoxedFailure = Result<(), BoxedError>;

pub struct Failed {
    details: String,
}

impl Failed {
    pub fn new(msg: &str) -> Self {
        Self { details: msg.to_string() }
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

pub fn throw<T>(message: String) -> Result<T, BoxedError> {
    Err(Box::new(Failed::new(&message)))
}

pub fn raise<T>(message: &str) -> Result<T, BoxedError> {
    Err(Box::new(Failed::new(&message)))
}

// endregion

#[cfg(test)]
mod tests {
    use crate::fails::*;

    #[test]
    fn check_throw_and_raise() {
        assert_eq!(throw::<usize>("fail".to_string()).is_ok(), false);
        assert_eq!(raise::<usize>("fail").is_ok(), false);
    }
}
