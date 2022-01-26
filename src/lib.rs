#![no_std]

pub mod ser;
pub mod de;
pub mod error;

pub mod size;

pub use de::deserialize;
// pub use error::{Error, Result};
pub use ser::serialize;
pub use size::SerializedSize;

pub use hubpack_derive::SerializedSize;

extern crate self as hubpack;

#[cfg(test)]
mod tests {
    use super::*;

    use serde::{Serialize, Deserialize};

    macro_rules! round_trip {
        ($testname:ident: $t:ty = $init:expr) => {
            #[test]
            fn $testname() {
                let input: $t = $init;
                const BUFSZ: usize = <$t as crate::SerializedSize>::MAX_SIZE;
                const PAD: usize = 3;
                let mut buffer: [u8; BUFSZ + PAD] = [0; BUFSZ + PAD];
                let len = serialize(&mut buffer, &input).unwrap();
                let (output, rest) = deserialize::<$t>(&buffer).unwrap();

                assert_eq!(output, input);
                assert_eq!(buffer.len() - rest.len(), len);
                assert!(len <= BUFSZ,
                    "Serialized length ({}) should be less than SerializedSize predicts ({})", len, BUFSZ);
            }
        }
    }

    round_trip!(rt_unit: () = ());

    round_trip!(rt_u8: u8 = 42);
    round_trip!(rt_u16: u16 = 0xDEAD);
    round_trip!(rt_u32: u32 = 0xDEADBEEF);
    round_trip!(rt_u64: u64 = 0xDEAD_BEEF_CAFE_D00D);
    round_trip!(rt_u128: u128 = 0xDEAD_BEEF_CAFE_D00D_1234_5678_ABCD_EF00);

    round_trip!(rt_i8: i8 = -42);
    round_trip!(rt_i16: i16 = -0x4EAD);
    round_trip!(rt_i32: i32 = -0x4EADBEEF);
    round_trip!(rt_i64: i64 = -0x4EAD_BEEF_CAFE_D00D);
    round_trip!(rt_i128: i128 = -0x4EAD_BEEF_CAFE_D00D_1234_5678_ABCD_EF00);

    round_trip!(rt_f32: f32 = core::f32::consts::PI);
    round_trip!(rt_f64: f64 = core::f64::consts::PI);

    round_trip!(rt_true: bool = true);
    round_trip!(rt_false: bool = false);

    round_trip!(rt_option_u8_none: Option<u8> = None);
    round_trip!(rt_option_u8_some: Option<u8> = Some(0xAA));

    round_trip!(rt_tuple: (u8, u16, bool) = (55, 0xCAFE, false));

    #[derive(Debug, Serialize, Deserialize, PartialEq, SerializedSize)]
    struct UnitStruct;

    round_trip!(rt_unit_struct: UnitStruct = UnitStruct);

    #[derive(Debug, Serialize, Deserialize, PartialEq, SerializedSize)]
    struct TupleStruct(u8, u32);

    round_trip!(rt_tuple_struct: TupleStruct = TupleStruct(12, 345678));

    #[derive(Debug, Serialize, Deserialize, PartialEq, SerializedSize)]
    struct Struct {
        a: Option<u16>,
        b: i16,
    }

    round_trip!(rt_struct: Struct = Struct { a: Some(0xF00D), b: -30 });

    #[derive(Debug, Serialize, Deserialize, PartialEq, SerializedSize)]
    enum Enum {
        Unit,
        Tuple(u8, u16),
        Struct {
            a: Option<u16>,
            b: i16,
        },
    }

    round_trip!(rt_enum_unit: Enum = Enum::Unit);
    round_trip!(rt_enum_tuple: Enum = Enum::Tuple(12, 3456));
    round_trip!(rt_enum_struct: Enum = Enum::Struct { a: Some(0xF00D), b: -12 });

    #[test]
    fn whither_usize() {
        let mut buf = [0; 8];
        let n = ser::serialize(&mut buf, &0usize).unwrap();
        assert_eq!(n, 8);
    }
}
