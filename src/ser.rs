//! Serialization of Rust values into `hubpack` format.

use serde::{ser, Serialize};
use crate::error::{Error, Result};

/// Serializes `value`, which must implement `serde::Serialize`, into `buf`.
/// On success, returns the number of bytes used.
///
/// Failures in `hubpack` itself fall into two groups.
///
/// - Dynamic failures: `Overrun`. This means that `buf` was not large enough to
///   contain the serialized representation of `value`, but a larger `buf` might
///   have succeeded.
/// - Static failures: `TooManyVariants` and `NotSupported`. These mean that the
///   type of `value` is simply incompatible with `hubpack` and won't work.
///
/// The catch-all error `Custom` may be produced by the `Serialize`
/// implementation of `value` or anything contained within `value`, but is never
/// produced by `hubpack` directly.
pub fn serialize(buf: &mut [u8], value: &impl Serialize) -> Result<usize> {
    let mut s = Serializer { buf, pos: 0 };
    value.serialize(&mut s)?;
    Ok(s.pos)
}

struct Serializer<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> Serializer<'a> {
    fn write_u8(&mut self, v: u8) -> Result<()> {
        *self.buf.get_mut(self.pos).ok_or(Error::Overrun)? = v;
        // We can use non-overflowing add here because the dereference using pos
        // just succeeded, meaning it is < buf.len, and buf.len can't be larger
        // than usize::MAX.
        self.pos = self.pos.wrapping_add(1);
        Ok(())
    }

    fn get_ary_mut<const N: usize>(&mut self) -> Result<&mut [u8; N]> {
        let chunk = self.buf.get_mut(self.pos..self.pos + N)
            .ok_or(Error::Overrun)?;
        // Restate the property of `get_mut` for the compiler. This helps avoid
        // generating unnecessary checks.
        assert!(chunk.len() == N);
        // We can use non-overflowing add here because the dereference using pos
        // just succeeded, meaning it is < buf.len, and buf.len can't be larger
        // than usize::MAX.
        self.pos = self.pos.wrapping_add(N);
        Ok(chunk.try_into().unwrap())
    }

    fn write_u16(&mut self, v: u16) -> Result<()> {
        *self.get_ary_mut()? = v.to_le_bytes();
        Ok(())
    }

    fn write_u32(&mut self, v: u32) -> Result<()> {
        *self.get_ary_mut()? = v.to_le_bytes();
        Ok(())
    }

    fn write_u64(&mut self, v: u64) -> Result<()> {
        *self.get_ary_mut()? = v.to_le_bytes();
        Ok(())
    }

    fn write_u128(&mut self, v: u128) -> Result<()> {
        *self.get_ary_mut()? = v.to_le_bytes();
        Ok(())
    }

    fn write_variant(&mut self, v: u32) -> Result<()> {
        self.write_u8(
            v.try_into().map_err(|_| Error::TooManyVariants)?
        )
    }
}

impl<'a, 'b> ser::Serializer for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    type SerializeSeq = ser::Impossible<(), Error>;
    type SerializeMap = ser::Impossible<(), Error>;

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.write_u8(v)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.write_u16(v)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.write_u32(v)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.write_u64(v)
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        self.write_u128(v)
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.write_u8(v as u8)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.write_u16(v as u16)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.write_u32(v as u32)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.write_u64(v as u64)
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        self.write_u128(v as u128)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.write_u32(v.to_bits())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.write_u64(v.to_bits())
    }

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.write_u8(u8::from(v))
    }

    fn serialize_char(self, v: char) -> Result<()> {
        if false {
            // As of the current Unicode version, the maximum UTF-8 encoded length
            // of any char is 4 bytes, which is also sizeof(char). So, that's handy.
            //
            // To ensure that any char value can encode, we require 4 bytes.
            // However, since we don't always consume all 4, we can't use the array
            // access routine.
            let dest = self.buf.get_mut(self.pos..self.pos + 4)
                .ok_or(Error::Overrun)?;
            let encoded = v.encode_utf8(dest);
            // Only advance by the required number of bytes.
            self.pos += encoded.len();
            Ok(())
        } else {
            return Err(Error::NotSupported);
        }
    }


    fn serialize_none(self) -> Result<()> {
        self.serialize_bool(false)
    }

    fn serialize_some<T: Serialize + ?Sized>(self, v: &T) -> Result<()> {
        self.serialize_bool(true)?;
        v.serialize(self)
    }


    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }


    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        self.write_variant(variant_index)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()> {
        self.write_variant(variant_index)?;
        value.serialize(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.write_variant(variant_index)?;
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.write_variant(variant_index)?;
        Ok(self)
    }


    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Ok(())
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(self, _name: &'static str, v: &T) -> Result<()> {
        v.serialize(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> Result<Self::SerializeSeq> {
        Err(Error::NotSupported)
    }

    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> Result<Self::SerializeMap> {
        Err(Error::NotSupported)
    }

    fn serialize_str(
        self,
        _v: &str,
    ) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn collect_str<T: ?Sized + core::fmt::Display>(
        self,
        _v: &T,
    ) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn serialize_bytes(
        self,
        _v: &[u8],
    ) -> Result<()> {
        Err(Error::NotSupported)
    }
}

impl<'a, 'b> ser::SerializeTuple for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: Serialize + ?Sized>(
        &mut self,
        element: &T,
    ) -> Result<()> {
        element.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, 'b> ser::SerializeTupleVariant for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        field: &T,
    ) -> Result<()> {
        field.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, 'b> ser::SerializeStructVariant for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        _key: &'static str,
        field: &T,
    ) -> Result<()> {
        field.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, 'b> ser::SerializeTupleStruct for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        field: &T,
    ) -> Result<()> {
        field.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, 'b> ser::SerializeStruct for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        _key: &'static str,
        field: &T,
    ) -> Result<()> {
        field.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

