use std::{error::Error, fmt::Display};

use crate::end::soft_quit;

#[derive(Debug, Clone, Copy)]
struct NoneError;

impl Display for NoneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NoneError")
    }
}

impl Error for NoneError {}

#[derive(Debug, Clone, Copy)]
struct FalseError;

impl Display for FalseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FalseError")
    }
}

impl Error for FalseError {}

fn die(err: impl Error, msg: &str) -> ! {
    soft_quit();
    panic!("{msg}: {err}");
}

pub trait OrDie<T> {
    fn or_die(self, msg: &str) -> T;
}

impl<T, E: Error> OrDie<T> for Result<T, E> {
    fn or_die(self, msg: &str) -> T {
        match self {
            Ok(value) => value,
            Err(err) => die(err, msg),
        }
    }
}

impl<T> OrDie<T> for Option<T> {
    fn or_die(self, msg: &str) -> T {
        match self {
            Some(value) => value,
            None => die(NoneError, msg),
        }
    }
}

impl OrDie<bool> for bool {
    fn or_die(self, msg: &str) -> bool {
        match self {
            true => true,
            false => die(FalseError, msg),
        }
    }
}
