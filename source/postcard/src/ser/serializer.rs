use std::{io::Write, marker::PhantomData};

use serde::{Serialize, ser};

use crate::{Cfg, FALSE, ID_COUNT, ID_LEN, ID_LEN_NAME, NONE, SOME, TRUE, UNKNOWN_LEN, varint::*};
use crate::{SPECIAL_LEN, cfg::DefaultCfg};
use crate::{
    error::{Error, Result},
    ser::skippable::SkipWrite,
};

/// Serializer
pub struct Serializer<W, CFG = DefaultCfg> {
    output: SkipWrite<W>,
    _cfg: PhantomData<CFG>,
}

impl<W: Write, CFG: Cfg> Serializer<W, CFG> {
    /// Creates a new serializer.
    pub fn new(write: W) -> Self {
        Self {
            output: SkipWrite::new(write),
            _cfg: PhantomData,
        }
    }

    /// Get the writer.
    pub fn finalize(self) -> W {
        self.output.into_inner()
    }

    fn write_usize(&mut self, data: usize) -> Result<()> {
        let value = u64::try_from(data).map_err(|_| Error::UsizeOverflow)?;
        self.write_u64(value)
    }

    fn write_u128(&mut self, data: u128) -> Result<()> {
        let mut buf = [0u8; varint_max::<u128>()];
        let used_buf = varint_u128(data, &mut buf);
        self.output.write(used_buf)?;
        Ok(())
    }

    fn write_u64(&mut self, data: u64) -> Result<()> {
        let mut buf = [0u8; varint_max::<u64>()];
        let used_buf = varint_u64(data, &mut buf);
        self.output.write(used_buf)?;
        Ok(())
    }

    fn write_u32(&mut self, data: u32) -> Result<()> {
        let mut buf = [0u8; varint_max::<u32>()];
        let used_buf = varint_u32(data, &mut buf);
        self.output.write(used_buf)?;
        Ok(())
    }

    fn write_u16(&mut self, data: u16) -> Result<()> {
        let mut buf = [0u8; varint_max::<u16>()];
        let used_buf = varint_u16(data, &mut buf);
        self.output.write(used_buf)?;
        Ok(())
    }

    fn write_identifier(&mut self, ident: &str) -> Result<()> {
        match ident
            .strip_prefix("_")
            .and_then(|s| s.parse::<usize>().ok())
        {
            Some(id) if id < ID_COUNT => {
                self.write_usize(ID_LEN_NAME + id)?;
            }
            _ => {
                let len = ident.len();
                if len < ID_LEN {
                    self.write_usize(len)?;
                } else {
                    self.write_usize(ID_LEN)?;
                    self.write_usize(len)?;
                }

                self.output.write(ident.as_bytes())?;
            }
        }

        Ok(())
    }
}

impl<'a, W, CFG> ser::Serializer for &'a mut Serializer<W, CFG>
where
    W: Write,
    CFG: Cfg,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SeqSerializer<'a, W, CFG>;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = MapSerializer<'a, W, CFG>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn is_human_readable(&self) -> bool {
        false
    }

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.serialize_u8(if v { TRUE } else { FALSE })
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_u8(v.to_le_bytes()[0])
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        let zzv = zig_zag_i16(v);
        self.write_u16(zzv)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        let zzv = zig_zag_i32(v);
        self.write_u32(zzv)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        let zzv = zig_zag_i64(v);
        self.write_u64(zzv)
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        let zzv = zig_zag_i128(v);
        self.write_u128(zzv)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        Ok(self.output.write(&[v])?)
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

    fn serialize_f32(self, v: f32) -> Result<()> {
        let buf = v.to_bits().to_le_bytes();
        Ok(self.output.write(&buf)?)
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        let buf = v.to_bits().to_le_bytes();
        Ok(self.output.write(&buf)?)
    }

    fn serialize_char(self, v: char) -> Result<()> {
        let mut buf = [0u8; 4];
        let strsl = v.encode_utf8(&mut buf);
        strsl.serialize(self)
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.write_usize(v.len())?;
        self.output.write(v.as_bytes())?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.write_usize(v.len())?;
        Ok(self.output.write(v)?)
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_u8(NONE)
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_u8(SOME)?;
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        if CFG::with_identifiers() {
            self.write_identifier(variant)?;
        } else {
            self.write_u32(variant_index)?;
        }
        Ok(())
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if CFG::with_identifiers() {
            self.write_identifier(variant)?;
        } else {
            self.write_u32(variant_index)?;
        }
        value.serialize(self)?;

        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        match len {
            Some(SPECIAL_LEN) => {
                self.write_usize(SPECIAL_LEN)?;
                self.write_usize(SPECIAL_LEN)?;
            }
            Some(len) => self.write_usize(len)?,
            None => {
                self.write_usize(SPECIAL_LEN)?;
                self.write_usize(UNKNOWN_LEN)?;
                self.output.start_skippable();
            }
        }

        Ok(SeqSerializer {
            serializer: self,
            len,
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        if CFG::with_identifiers() {
            self.write_identifier(variant)?;
        } else {
            self.write_u32(variant_index)?;
        }

        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        match len {
            Some(SPECIAL_LEN) => {
                self.write_usize(SPECIAL_LEN)?;
                self.write_usize(SPECIAL_LEN)?;
            }
            Some(len) => self.write_usize(len)?,
            None => {
                self.write_usize(SPECIAL_LEN)?;
                self.write_usize(UNKNOWN_LEN)?;
                self.output.start_skippable();
            }
        }

        Ok(MapSerializer {
            serializer: self,
            len,
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.write_usize(len)?;

        if !CFG::with_identifiers() {
            self.output.start_skippable();
        }

        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        if CFG::with_identifiers() {
            self.write_identifier(variant)?;
        } else {
            self.write_u32(variant_index)?;
        }

        self.write_usize(len)?;

        if !CFG::with_identifiers() {
            self.output.start_skippable();
        }

        Ok(self)
    }
}

pub struct SeqSerializer<'a, W, CFG> {
    serializer: &'a mut Serializer<W, CFG>,
    len: Option<usize>,
}

impl<'a, W, CFG> ser::SerializeSeq for SeqSerializer<'a, W, CFG>
where
    W: Write,
    CFG: Cfg,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<()> {
        if self.len.is_none() {
            self.serializer.output.end_skippable()?;
        }

        Ok(())
    }
}

impl<W, CFG> ser::SerializeTuple for &mut Serializer<W, CFG>
where
    W: Write,
    CFG: Cfg,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<W, CFG> ser::SerializeTupleStruct for &mut Serializer<W, CFG>
where
    W: Write,
    CFG: Cfg,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<W, CFG> ser::SerializeTupleVariant for &mut Serializer<W, CFG>
where
    W: Write,
    CFG: Cfg,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

pub struct MapSerializer<'a, W, CFG> {
    serializer: &'a mut Serializer<W, CFG>,
    len: Option<usize>,
}

impl<'a, W, CFG> ser::SerializeMap for MapSerializer<'a, W, CFG>
where
    W: Write,
    CFG: Cfg,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut *self.serializer)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<()> {
        if self.len.is_none() {
            self.serializer.output.end_skippable()?;
        }

        Ok(())
    }
}

impl<W, CFG> ser::SerializeStruct for &mut Serializer<W, CFG>
where
    W: Write,
    CFG: Cfg,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if CFG::with_identifiers() {
            self.write_identifier(key)?;
            self.output.start_skippable();
        }

        value.serialize(&mut **self)?;

        if CFG::with_identifiers() {
            self.output.end_skippable()?;
        }

        Ok(())
    }

    fn end(self) -> Result<()> {
        if !CFG::with_identifiers() {
            self.output.end_skippable()?;
        }

        Ok(())
    }
}

impl<W, CFG> ser::SerializeStructVariant for &mut Serializer<W, CFG>
where
    W: Write,
    CFG: Cfg,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if CFG::with_identifiers() {
            self.write_identifier(key)?;
            self.output.start_skippable();
        }

        value.serialize(&mut **self)?;

        if CFG::with_identifiers() {
            self.output.end_skippable()?;
        }

        Ok(())
    }

    fn end(self) -> Result<()> {
        if !CFG::with_identifiers() {
            self.output.end_skippable()?;
        }

        Ok(())
    }
}

fn zig_zag_i16(n: i16) -> u16 {
    ((n << 1) ^ (n >> 15)) as u16
}

fn zig_zag_i32(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

fn zig_zag_i64(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

fn zig_zag_i128(n: i128) -> u128 {
    ((n << 1) ^ (n >> 127)) as u128
}
