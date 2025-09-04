use serde::{Serialize, ser};
use std::{io::Write, marker::PhantomData};

use crate::{Cfg, UNKNOWN_LEN, varint::*};
use crate::{SPECIAL_LEN, cfg::DefaultCfg};
use crate::{
    error::{Error, Result},
    ser::skippable::SkipWrite,
};

/// A `serde` compatible serializer, generic over "Flavors" of serializing plugins.
///
/// It should rarely be necessary to directly use this type unless you are implementing your
/// own [`SerFlavor`].
///
/// See the docs for [`SerFlavor`] for more information about "flavors" of serialization
///
/// [`SerFlavor`]: crate::ser_flavors::Flavor
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

    /// Get the writer after flushing it.
    pub fn finalize(self) -> Result<W> {
        Ok(self.output.into_inner()?)
    }

    /// Attempt to push a variably encoded [usize] into the output data stream
    pub(crate) fn write_usize(&mut self, data: usize) -> Result<()> {
        let value = u64::try_from(data).map_err(|_| Error::UsizeOverflow)?;
        self.write_u64(value)
    }

    /// Attempt to push a variably encoded [u128] into the output data stream
    pub(crate) fn write_u128(&mut self, data: u128) -> Result<()> {
        let mut buf = [0u8; varint_max::<u128>()];
        let used_buf = varint_u128(data, &mut buf);
        self.output.write(used_buf)?;
        Ok(())
    }

    /// Attempt to push a variably encoded [u64] into the output data stream
    pub(crate) fn write_u64(&mut self, data: u64) -> Result<()> {
        let mut buf = [0u8; varint_max::<u64>()];
        let used_buf = varint_u64(data, &mut buf);
        self.output.write(used_buf)?;
        Ok(())
    }

    /// Attempt to push a variably encoded [u32] into the output data stream
    pub(crate) fn write_u32(&mut self, data: u32) -> Result<()> {
        let mut buf = [0u8; varint_max::<u32>()];
        let used_buf = varint_u32(data, &mut buf);
        self.output.write(used_buf)?;
        Ok(())
    }

    /// Attempt to push a variably encoded [u16] into the output data stream
    pub(crate) fn write_u16(&mut self, data: u16) -> Result<()> {
        let mut buf = [0u8; varint_max::<u16>()];
        let used_buf = varint_u16(data, &mut buf);
        self.output.write(used_buf)?;
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
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn is_human_readable(&self) -> bool {
        false
    }

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.serialize_u8(if v { 1 } else { 0 })
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
        self.serialize_u8(0)
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_u8(1)?;
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
            variant.serialize(&mut *self)?;
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
            variant.serialize(&mut *self)?;
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
            variant.serialize(&mut *self)?;
        } else {
            self.write_u32(variant_index)?;
        }

        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.write_usize(len.ok_or(Error::SerializeSeqLengthUnknown)?)?;
        Ok(self)
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
            variant.serialize(&mut *self)?;
        } else {
            self.write_u32(variant_index)?;
        }

        self.write_usize(len)?;

        if !CFG::with_identifiers() {
            self.output.start_skippable();
        }

        Ok(self)
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: core::fmt::Display + ?Sized,
    {
        use core::fmt::Write;

        // Unfortunately, we need to know the size of the serialized data before
        // we can place it into the output. In order to do this, we run the formatting
        // of the output data TWICE, the first time to determine the length, the
        // second time to actually format the data
        //
        // There are potentially other ways to do this, such as:
        //
        // * Reserving a fixed max size, such as 5 bytes, for the length field, and
        //     leaving non-canonical trailing zeroes at the end. This would work up
        //     to some reasonable length, but might have some portability vs max size
        //     tradeoffs, e.g. 64KiB if we pick 3 bytes, or 4GiB if we pick 5 bytes
        // * Expose some kind of "memmove" capability to flavors, to allow us to
        //     format into the buffer, then "scoot over" that many times.
        //
        // Despite the current approaches downside in speed, it is likely flexible
        // enough for the rare-ish case where formatting a Debug impl is necessary.
        // This is better than the previous panicking behavior, and can be improved
        // in the future.
        struct CountWriter {
            ct: usize,
        }
        impl Write for CountWriter {
            fn write_str(&mut self, s: &str) -> core::result::Result<(), core::fmt::Error> {
                self.ct += s.len();
                Ok(())
            }
        }

        let mut ctr = CountWriter { ct: 0 };

        // This is the first pass through, where we just count the length of the
        // data that we are given
        write!(&mut ctr, "{value}").map_err(|_| Error::CollectStrError)?;
        let len = ctr.ct;
        self.write_usize(len)?;

        struct FmtWriter<'a, IW>
        where
            IW: std::io::Write,
        {
            output: &'a mut SkipWrite<IW>,
        }
        impl<IW> Write for FmtWriter<'_, IW>
        where
            IW: std::io::Write,
        {
            fn write_str(&mut self, s: &str) -> core::result::Result<(), core::fmt::Error> {
                self.output
                    .write(s.as_bytes())
                    .map_err(|_| core::fmt::Error)
            }
        }

        // This second pass actually inserts the data.
        let mut fw = FmtWriter {
            output: &mut self.output,
        };
        write!(&mut fw, "{value}").map_err(|_| Error::CollectStrError)?;

        Ok(())
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

impl<W, CFG> ser::SerializeMap for &mut Serializer<W, CFG>
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
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
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
            key.serialize(&mut **self)?;
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
            key.serialize(&mut **self)?;
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
