use std::fmt::{Debug, Display};

pub(crate) enum Error {
    Custom(String),
    Io(std::io::Error),
}

macro_rules! impl_from {
    ($from:ty => $variant:ident) => {
        impl From<$from> for Error {
            fn from(value: $from) -> Self {
                Error::$variant(value)
            }
        }
    };
}

impl_from!(String => Custom);
impl_from!(std::io::Error => Io);

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Custom(e) => write!(f, "{}", e),
            Error::Io(e) => write!(f, "Filesystem operation failed: {}", e),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
