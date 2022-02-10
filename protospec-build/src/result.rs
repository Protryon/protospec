use std::fmt;
pub use std::io::ErrorKind;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type Result<T> = std::result::Result<T, Error>;
pub type StdResult<T, E> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct ProtoSpecError {
    message: String,
}

impl ProtoSpecError {
    pub fn new(message: String) -> ProtoSpecError {
        ProtoSpecError { message }
    }
}

impl fmt::Display for ProtoSpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProtoSpecError {{ {} }}", self.message)
    }
}

impl std::error::Error for ProtoSpecError {}

#[macro_export]
macro_rules! protospec_err {
    ($($arg:tt)*) => { Box::new(ProtoSpecError::new(format!($($arg)*))) }
}
