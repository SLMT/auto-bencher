
#[derive(Debug)]
pub enum BenchError {
    NoSuchCommand,
    CommandFails(i32),
    FileNotFound,
    Throw(String)
}

impl BenchError {
    pub fn throw<E>(message: &str, err: E) -> BenchError
        where E: ToString {
        BenchError::Throw(
           format!("{}: {}", message, err.to_string())
        )
    }

    pub fn message(message: &str) -> BenchError {
        BenchError::Throw(
           format!("{}", message)
        )
    }
}