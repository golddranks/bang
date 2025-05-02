use std::fmt::Display;

#[must_use]
pub struct EndMsg<M> {
    pub callback: M,
    pub file: &'static str,
    pub line: u32,
}

impl<M> EndMsg<M>
where
    M: FnOnce() -> String,
{
    pub fn end_with(self, causing_err: impl Display) -> ! {
        panic!(
            "[{}:{}]: {}: {}",
            self.file,
            self.line,
            (self.callback)(),
            causing_err
        );
    }

    pub fn new(callback: M, file: &'static str, line: u32) -> Self {
        Self {
            file,
            line,
            callback,
        }
    }
}

#[macro_export]
macro_rules! die_now {
    () => {
        $crate::error::EndMsg::new(|| "".to_string(), file!(), line!()).end_with("")
    };
    ($fmt:literal $(, $args:expr)*) => {
        $crate::error::EndMsg::new(|| format!($fmt, $($args),*), file!(), line!()).end_with("")
    };
}

#[macro_export]
macro_rules! die {
    () => {
        $crate::error::EndMsg::new(|| "".to_string(), file!(), line!())
    };
    ($fmt:literal $(, $args:expr)*) => {
        $crate::error::EndMsg::new(|| format!($fmt, $($args),*), file!(), line!())
    };
}

pub trait OrDie<T> {
    fn or_<M>(self, end_msg: EndMsg<M>) -> T
    where
        M: FnOnce() -> String;
}

impl<T, E> OrDie<T> for Result<T, E>
where
    E: Display,
{
    fn or_<M>(self, end_msg: EndMsg<M>) -> T
    where
        M: FnOnce() -> String,
    {
        match self {
            Ok(t) => t,
            Err(e) => end_msg.end_with(e),
        }
    }
}

impl<T> OrDie<T> for Option<T> {
    fn or_<M>(self, end_msg: EndMsg<M>) -> T
    where
        M: FnOnce() -> String,
    {
        match self {
            Some(t) => t,
            None => end_msg.end_with(""),
        }
    }
}

impl OrDie<bool> for bool {
    fn or_<M>(self, end_msg: EndMsg<M>) -> bool
    where
        M: FnOnce() -> String,
    {
        match self {
            true => true,
            false => end_msg.end_with(""),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[derive(Debug)]
    struct TestError;

    impl Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TestError")
        }
    }

    impl Error for TestError {}

    #[test]
    #[should_panic]
    fn test_error_err() {
        Err::<(), _>(TestError).or_(die!("test_error_err"));
    }

    #[test]
    #[should_panic]
    fn test_error_none() {
        Option::<()>::None.or_(die!("test_error_none"));
    }

    #[test]
    #[should_panic]
    fn test_error_false() {
        false.or_(die!("test_error_false"));
    }

    #[test]
    fn test_error_truish() {
        Ok::<(), TestError>(()).or_(die!("test_error_truish"));
        Some(()).or_(die!("test_error_truish"));
        true.or_(die!("test_error_truish"));
    }
}
