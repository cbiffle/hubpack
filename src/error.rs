#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Error {
    Custom,
    Overrun,
    TooManyVariants,
    NotSupported,
    Truncated,
    Invalid,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Custom => f.write_str("Custom"),
            Self::Overrun => f.write_str("serialization buffer too small"),
            Self::TooManyVariants => f.write_str("too many enum variants (format only supports 256)"),
            Self::NotSupported => f.write_str("type not supported"),
            Self::Truncated => f.write_str("truncated"),
            Self::Invalid => f.write_str("invalid/corrupt"),
        }
    }
}

impl serde::ser::Error for Error {
    fn custom<T: core::fmt::Display>(_msg: T) -> Self {
        Self::Custom
    }
}

impl serde::de::Error for Error {
    fn custom<T: core::fmt::Display>(_msg: T) -> Self {
        Self::Custom
    }
}

// Allow our use by crates that have serde's `std` feature enabled. serde
// reexports `StdError` under both `serde::ser` and `serde::de`; we just have to
// pick one.
impl serde::ser::StdError for Error {}

pub type Result<T> = core::result::Result<T, Error>;

