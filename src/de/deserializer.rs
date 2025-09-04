use std::{io::Read, marker::PhantomData};

use serde::de::{
    self, DeserializeSeed, IntoDeserializer, Visitor,
    value::{StringDeserializer, U32Deserializer},
};

use crate::{
    Cfg, FALSE, ID_COUNT, ID_LEN, ID_LEN_NAME, NONE, SOME, SPECIAL_LEN, TRUE, UNKNOWN_LEN,
    cfg::DefaultCfg,
    de::skippable::SkipRead,
    error::{Error, Result},
    varint::{max_of_last_byte, varint_max},
};

/// Deserializer.
pub struct Deserializer<'de, R, CFG = DefaultCfg> {
    input: SkipRead<R>,
    _de: PhantomData<&'de ()>,
    _cfg: PhantomData<CFG>,
}

impl<'de, R, CFG: Cfg> Deserializer<'de, R, CFG>
where
    R: Read,
{
    /// Obtain a Deserializer from a reader.
    pub fn new(read: R) -> Self {
        Deserializer { input: SkipRead::new(read), _de: PhantomData, _cfg: PhantomData }
    }

    /// Returns the reader.
    pub fn finalize(self) -> R {
        self.input.into_inner()
    }
}

impl<'de, R: Read, CFG: Cfg> Deserializer<'de, R, CFG> {
    fn try_take_varint_usize(&mut self) -> Result<usize> {
        let value = self.read_varint_u64()?;
        usize::try_from(value).map_err(|_| Error::UsizeOverflow)
    }

    fn read_varint_u16(&mut self) -> Result<u16> {
        let mut out = 0;
        for i in 0..varint_max::<u16>() {
            let val = self.input.read_u8()?;
            let carry = (val & 0x7F) as u16;
            out |= carry << (7 * i);

            if (val & 0x80) == 0 {
                if i == varint_max::<u16>() - 1 && val > max_of_last_byte::<u16>() {
                    return Err(Error::BadVarint);
                } else {
                    return Ok(out);
                }
            }
        }
        Err(Error::BadVarint)
    }

    fn read_varint_u32(&mut self) -> Result<u32> {
        let mut out = 0;
        for i in 0..varint_max::<u32>() {
            let val = self.input.read_u8()?;
            let carry = (val & 0x7F) as u32;
            out |= carry << (7 * i);

            if (val & 0x80) == 0 {
                if i == varint_max::<u32>() - 1 && val > max_of_last_byte::<u32>() {
                    return Err(Error::BadVarint);
                } else {
                    return Ok(out);
                }
            }
        }
        Err(Error::BadVarint)
    }

    fn read_varint_u64(&mut self) -> Result<u64> {
        let mut out = 0;
        for i in 0..varint_max::<u64>() {
            let val = self.input.read_u8()?;
            let carry = (val & 0x7F) as u64;
            out |= carry << (7 * i);

            if (val & 0x80) == 0 {
                if i == varint_max::<u64>() - 1 && val > max_of_last_byte::<u64>() {
                    return Err(Error::BadVarint);
                } else {
                    return Ok(out);
                }
            }
        }
        Err(Error::BadVarint)
    }

    fn read_varint_u128(&mut self) -> Result<u128> {
        let mut out = 0;
        for i in 0..varint_max::<u128>() {
            let val = self.input.read_u8()?;
            let carry = (val & 0x7F) as u128;
            out |= carry << (7 * i);

            if (val & 0x80) == 0 {
                if i == varint_max::<u128>() - 1 && val > max_of_last_byte::<u128>() {
                    return Err(Error::BadVarint);
                } else {
                    return Ok(out);
                }
            }
        }
        Err(Error::BadVarint)
    }

    fn read_identifier(&mut self) -> Result<String> {
        let v = self.try_take_varint_usize()?;

        if v >= ID_LEN_NAME + ID_COUNT {
            return Err(Error::BadIdentifier);
        }

        if v >= ID_LEN_NAME {
            let id = v - ID_LEN_NAME;
            return Ok(format!("_{id}"));
        }

        let len = if v == ID_LEN { self.try_take_varint_usize()? } else { v };

        let bytes = self.input.read(len)?;
        String::from_utf8(bytes).map_err(|_| Error::BadIdentifier)
    }
}

struct SeqAccess<'a, 'b, R, CFG> {
    deserializer: &'a mut Deserializer<'b, R, CFG>,
    len: Option<usize>,
}

impl<'a, 'b: 'a, R: Read, CFG: Cfg> serde::de::SeqAccess<'b> for SeqAccess<'a, 'b, R, CFG> {
    type Error = Error;

    fn next_element_seed<V: DeserializeSeed<'b>>(&mut self, seed: V) -> Result<Option<V::Value>> {
        match &mut self.len {
            Some(0) => Ok(None),
            Some(len) => {
                *len -= 1;
                let data = DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
                Ok(Some(data))
            }
            None => match DeserializeSeed::deserialize(seed, &mut *self.deserializer) {
                Ok(data) => Ok(Some(data)),
                Err(Error::EndOfBlock) => Ok(None),
                Err(err) => Err(err),
            },
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.len
    }
}

struct StructSeqAccess<'a, 'b, R, CFG> {
    deserializer: &'a mut Deserializer<'b, R, CFG>,
    len: usize,
}

impl<'a, 'b: 'a, R: Read, CFG: Cfg> serde::de::SeqAccess<'b> for StructSeqAccess<'a, 'b, R, CFG> {
    type Error = Error;

    fn next_element_seed<V: DeserializeSeed<'b>>(&mut self, seed: V) -> Result<Option<V::Value>> {
        assert!(!CFG::with_identifiers());

        if self.len > 0 {
            self.len -= 1;
            let data = DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

struct StructFieldAccess<'a, 'b, R, CFG> {
    deserializer: &'a mut Deserializer<'b, R, CFG>,
    len: usize,
}

impl<'a, 'b: 'a, R: Read, CFG: Cfg> serde::de::MapAccess<'b> for StructFieldAccess<'a, 'b, R, CFG> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'b>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        if self.len > 0 {
            self.len -= 1;
            let value = DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V: DeserializeSeed<'b>>(&mut self, seed: V) -> Result<V::Value> {
        assert!(CFG::with_identifiers());

        self.deserializer.input.start_skippable();
        let value = DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
        self.deserializer.input.end_skippable()?;

        Ok(value)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

struct MapAccess<'a, 'b, R, CFG> {
    deserializer: &'a mut Deserializer<'b, R, CFG>,
    len: Option<usize>,
}

impl<'a, 'b: 'a, R: Read, CFG: Cfg> serde::de::MapAccess<'b> for MapAccess<'a, 'b, R, CFG> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'b>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        match &mut self.len {
            Some(0) => Ok(None),
            Some(len) => {
                *len -= 1;
                let data = DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
                Ok(Some(data))
            }
            None => match DeserializeSeed::deserialize(seed, &mut *self.deserializer) {
                Ok(data) => Ok(Some(data)),
                Err(Error::EndOfBlock) => Ok(None),
                Err(err) => Err(err),
            },
        }
    }

    fn next_value_seed<V: DeserializeSeed<'b>>(&mut self, seed: V) -> Result<V::Value> {
        DeserializeSeed::deserialize(seed, &mut *self.deserializer)
    }

    fn size_hint(&self) -> Option<usize> {
        self.len
    }
}

impl<'de, R: Read, CFG: Cfg> de::Deserializer<'de> for &mut Deserializer<'de, R, CFG> {
    type Error = Error;

    fn is_human_readable(&self) -> bool {
        false
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::DeserializeAnyUnsupported)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let val = match self.input.read_u8()? {
            FALSE => false,
            TRUE => true,
            _ => return Err(Error::BadBool),
        };
        visitor.visit_bool(val)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.input.read_u8()? as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_varint_u16()?;
        visitor.visit_i16(de_zig_zag_i16(v))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_varint_u32()?;
        visitor.visit_i32(de_zig_zag_i32(v))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_varint_u64()?;
        visitor.visit_i64(de_zig_zag_i64(v))
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_varint_u128()?;
        visitor.visit_i128(de_zig_zag_i128(v))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.input.read_u8()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_varint_u16()?;
        visitor.visit_u16(v)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_varint_u32()?;
        visitor.visit_u32(v)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_varint_u64()?;
        visitor.visit_u64(v)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_varint_u128()?;
        visitor.visit_u128(v)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.input.read(4)?;
        visitor.visit_f32(f32::from_bits(u32::from_le_bytes(bytes.try_into().unwrap())))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.input.read(8)?;
        visitor.visit_f64(f64::from_bits(u64::from_le_bytes(bytes.try_into().unwrap())))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let sz = self.try_take_varint_usize()?;
        if sz > 4 {
            return Err(Error::BadChar);
        }
        let bytes = self.input.read(sz)?;

        let character =
            core::str::from_utf8(&bytes).map_err(|_| Error::BadChar)?.chars().next().ok_or(Error::BadChar)?;
        visitor.visit_char(character)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let sz = self.try_take_varint_usize()?;
        let bytes = self.input.read(sz)?;
        let str_sl = String::from_utf8(bytes).map_err(|_| Error::BadString)?;

        visitor.visit_string(str_sl)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let sz = self.try_take_varint_usize()?;
        let bytes = self.input.read(sz)?;
        visitor.visit_byte_buf(bytes)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.input.read_u8()? {
            NONE => visitor.visit_none(),
            SOME => visitor.visit_some(self),
            _ => Err(Error::BadOption),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = match self.try_take_varint_usize()? {
            SPECIAL_LEN => match self.try_take_varint_usize()? {
                SPECIAL_LEN => Some(SPECIAL_LEN),
                UNKNOWN_LEN => {
                    self.input.start_skippable();
                    None
                }
                _ => return Err(Error::BadLen),
            },
            len => Some(len),
        };

        let value = visitor.visit_seq(SeqAccess { deserializer: self, len })?;

        if len.is_none() {
            self.input.end_skippable()?;
        }

        Ok(value)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqAccess { deserializer: self, len: Some(len) })
    }

    fn deserialize_tuple_struct<V>(self, _name: &'static str, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = match self.try_take_varint_usize()? {
            SPECIAL_LEN => match self.try_take_varint_usize()? {
                SPECIAL_LEN => Some(SPECIAL_LEN),
                UNKNOWN_LEN => {
                    self.input.start_skippable();
                    None
                }
                _ => return Err(Error::BadLen),
            },
            len => Some(len),
        };

        let value = visitor.visit_map(MapAccess { deserializer: self, len })?;

        if len.is_none() {
            self.input.end_skippable()?;
        }

        Ok(value)
    }

    fn deserialize_struct<V>(
        self, _name: &'static str, _fields: &'static [&'static str], visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.try_take_varint_usize()?;

        if CFG::with_identifiers() {
            visitor.visit_map(StructFieldAccess { deserializer: self, len })
        } else {
            self.input.start_skippable();
            let value = visitor.visit_seq(StructSeqAccess { deserializer: self, len })?;
            self.input.end_skippable()?;
            Ok(value)
        }
    }

    fn deserialize_enum<V>(
        self, _name: &'static str, _variants: &'static [&'static str], visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.read_identifier()?)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

impl<'de, R: Read, CFG: Cfg> serde::de::VariantAccess<'de> for &mut Deserializer<'de, R, CFG> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<V::Value> {
        DeserializeSeed::deserialize(seed, self)
    }

    fn tuple_variant<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        serde::de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    fn struct_variant<V: Visitor<'de>>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value> {
        serde::de::Deserializer::deserialize_struct(self, "", _fields, visitor)
    }
}

impl<'de, R: Read, CFG: Cfg> serde::de::EnumAccess<'de> for &mut Deserializer<'de, R, CFG> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self)> {
        let v = if CFG::with_identifiers() {
            let ident = self.read_identifier()?;
            let deserializer: StringDeserializer<Error> = ident.into_deserializer();
            DeserializeSeed::deserialize(seed, deserializer)?
        } else {
            let varint = self.read_varint_u32()?;
            let deserializer: U32Deserializer<Error> = varint.into_deserializer();
            DeserializeSeed::deserialize(seed, deserializer)?
        };

        Ok((v, self))
    }
}

fn de_zig_zag_i16(n: u16) -> i16 {
    ((n >> 1) as i16) ^ (-((n & 0b1) as i16))
}

fn de_zig_zag_i32(n: u32) -> i32 {
    ((n >> 1) as i32) ^ (-((n & 0b1) as i32))
}

fn de_zig_zag_i64(n: u64) -> i64 {
    ((n >> 1) as i64) ^ (-((n & 0b1) as i64))
}

fn de_zig_zag_i128(n: u128) -> i128 {
    ((n >> 1) as i128) ^ (-((n & 0b1) as i128))
}
