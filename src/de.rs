//! Deserializing `hubpack`-encoded values back into Rust.

use serde::de::{self, Visitor, IntoDeserializer};
use serde::Deserialize;
use crate::error::{Error, Result};

/// Deserializes a `T` from the serialized representation at the start of
/// `data`. Deserialization may succeed even if there's additional data tacked
/// onto the end of the `hubpack` serialized representation.
///
/// On success, this returns two values:
/// - The `T` that was deserialized, and
/// - The rest of `data`, as a slice.
///
/// The remaining, unused `data` is returned as a slice, rather than a number of
/// bytes consumed or remaining, so that you can use the rest of the data
/// without incurring a slice bounds check.
///
/// Note that the `hubpack` format is explicitly designed to allow multiple
/// serialized values to be simply concatenated together and then deserialized
/// correctly.
pub fn deserialize<T: de::DeserializeOwned>(data: &[u8]) -> Result<(T, &[u8])> {
    let mut d = Deserializer { data };
    let val = T::deserialize(&mut d)?;
    Ok((val, d.data))
}

struct Deserializer<'de> {
    data: &'de [u8],
}

impl<'de> Deserializer<'de> {
    fn take_u8(&mut self) -> Result<u8> {
        let (first, rest) = self.data.split_first()
            .ok_or(Error::Truncated)?;
        self.data = rest;
        Ok(*first)
    }

    fn take_ary<const N: usize>(&mut self) -> Result<[u8; N]> {
        if N <= self.data.len() {
            let (chunk, rest) = self.data.split_at(N);
            self.data = rest;
            Ok(chunk.try_into().unwrap())
        } else {
            Err(Error::Truncated)
        }
    }

    fn take_u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take_ary()?))
    }

    fn take_u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take_ary()?))
    }

    fn take_u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take_ary()?))
    }

    fn take_u128(&mut self) -> Result<u128> {
        Ok(u128::from_le_bytes(self.take_ary()?))
    }

}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.take_u8()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.take_u16()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.take_u32()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.take_u64()?)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u128(self.take_u128()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.take_u8()? as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.take_u16()? as i16)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.take_u32()? as i32)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.take_u64()? as i64)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i128(self.take_u128()? as i128)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(f32::from_bits(self.take_u32()?))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(f64::from_bits(self.take_u64()?))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.take_u8()? {
            0 => visitor.visit_bool(false),
            1 => visitor.visit_bool(true),
            _ => Err(Error::Invalid),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if bool::deserialize(&mut *self)? {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqAccess { inner: self, len: len })
    }

    fn deserialize_tuple_struct<V>(self, _name: &'static str, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_struct<V>(self, _name: &'static str, fields: &[&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(self, _name: &'static str, _variants: &[&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::NotSupported)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::NotSupported)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::NotSupported)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::NotSupported)
    }

    fn deserialize_str<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::NotSupported)
    }

    fn deserialize_string<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::NotSupported)
    }

    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::NotSupported)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::NotSupported)
    }

    fn deserialize_map<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::NotSupported)
    }

    fn deserialize_char<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::NotSupported)
    }

}

struct SeqAccess<'a, 'de> {
    inner: &'a mut Deserializer<'de>,
    len: usize,
}

impl<'a, 'de> de::SeqAccess<'de> for SeqAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>> {
        if let Some(new_len) = self.len.checked_sub(1) {
            self.len = new_len;
            Ok(Some(seed.deserialize(&mut *self.inner)?))
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

impl<'b, 'de> de::EnumAccess<'de> for &'b mut Deserializer<'de> {
    type Error = Error;
    type Variant = Self;
    
    fn variant_seed<V: de::DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self)> {
        let v = u8::deserialize(&mut *self)?;
        Ok((seed.deserialize(u32::from(v).into_deserializer())?, self))
    }
}

impl<'b, 'de> de::VariantAccess<'de> for &'b mut Deserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T: de::DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        seed.deserialize(self)
    }

    fn tuple_variant<V: de::Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        serde::Deserializer::deserialize_tuple(self, len, visitor)
    }

    fn struct_variant<V: de::Visitor<'de>>(self, fields: &[&'static str], visitor: V) -> Result<V::Value> {
        serde::Deserializer::deserialize_tuple(self, fields.len(), visitor)
    }
}

