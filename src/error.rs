//! Error type.

/// Errors that can result from `hubpack` serialization or deserialization.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The `Serialize` or `Deserialize` implementation for some type produced
    /// `serde`'s `Custom` error type. This is never produced by `hubpack`
    /// itself, but is simply passed through from existing types.
    Custom,
    /// Serializing a value failed because there were not enough bytes
    /// available in the destination buffer.
    Overrun,
    /// Serializing a value failed because it is an enum type with more than 256
    /// variants, which we don't support.
    TooManyVariants,
    /// Serializing a value failed because it is a type we don't support, such
    /// as a sequence, map, or `char`.
    NotSupported,
    /// Deserializing a value failed because its serialized representation ended
    /// unexpectedly.
    Truncated,
    /// Deserializing a value failed because an encoded value was out of range
    /// for its type, such as a `bool` with a value of `39`.
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

