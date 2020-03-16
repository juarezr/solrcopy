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

pub fn throw<T>(message: String) -> BoxedResult<T> {
    Err(Box::new(Failed::new(&message)))
}

pub fn rethrow<T, E>(failure: E) -> BoxedResult<T>
where
    E: Error + 'static,
{
    Err(Box::new(failure))
}

pub fn raise<T>(message: &str) -> BoxedResult<T> {
    Err(Box::new(Failed::new(&message)))
}

// endregion

#[cfg(test)]
mod tests {
    use crate::fails::*;

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
