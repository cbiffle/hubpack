#![no_std]

pub mod ser;
pub mod de;
pub mod error;

pub mod size;

pub use de::deserialize;
// pub use error::{Error, Result};
pub use ser::serialize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whither_usize() {
        let mut buf = [0; 8];
        let n = ser::serialize(&mut buf, &0usize).unwrap();
        assert_eq!(n, 8);
         
    }
}
