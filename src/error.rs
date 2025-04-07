pub trait OrDie<T> {
    fn or_die(self, msg: &str) -> T;
}

impl<T, E> OrDie<T> for Result<T, E> {
    fn or_die(self, msg: &str) -> T {
        match self {
            Ok(value) => value,
            Err(_) => panic!("{}", msg),
        }
    }
}

impl<T> OrDie<T> for Option<T> {
    fn or_die(self, msg: &str) -> T {
        match self {
            Some(value) => value,
            None => panic!("{}", msg),
        }
    }
}
