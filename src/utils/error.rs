use std::fmt::Formatter;

#[derive(Debug)]
pub enum Z2pError {
    Unauthorized,
    BadRequest,
}

impl std::fmt::Display for Z2pError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Z2pError::Unauthorized => write!(f, "Unauthorized"),
            Z2pError::BadRequest => write!(f, "BadRequest")
        }
    }
}
