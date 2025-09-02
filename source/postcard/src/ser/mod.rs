use crate::ser::flavors::Flavor;
use crate::{
    error::{Error, Result},
    ser::skippable::SkipStack,
};
use serde::Serialize;

use crate::ser::serializer::Serializer;

pub mod flavors;
pub(crate) mod serializer;
pub(crate) mod skippable;

/// Serialize a `T` to a `std::vec::Vec<u8>`.
///
/// ## Example
///
/// ```rust
/// use postcard::to_vec;
///
/// let ser: Vec<u8> = to_vec(&true).unwrap();
/// assert_eq!(ser.as_slice(), &[0x01]);
///
/// let ser: Vec<u8> = to_vec("Hi!").unwrap();
/// assert_eq!(ser.as_slice(), &[0x03, b'H', b'i', b'!']);
/// ```
#[cfg(feature = "use-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "use-std")))]
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    to_io(value, std::vec::Vec::new())
}

/// Serialize a `T` to a [`std::io::Write`],
/// ## Example
///
/// ```rust
/// use postcard::to_io;
/// let mut buf: [u8; 32] = [0; 32];
/// let mut writer: &mut [u8] = &mut buf;
///
/// let ser = to_io(&true, &mut writer).unwrap();
/// to_io("Hi!", ser).unwrap();
/// assert_eq!(&buf[0..5], &[0x01, 0x03, b'H', b'i', b'!']);
/// ```
#[cfg(feature = "use-std")]
pub fn to_io<T, W>(value: &T, writer: W) -> Result<W>
where
    T: Serialize + ?Sized,
    W: std::io::Write,
{
    serialize_with_flavor::<T, _, _>(value, flavors::io::WriteFlavor::new(writer))
}

/// `serialize_with_flavor()` has three generic parameters, `T, F, O`.
///
/// * `T`: This is the type that is being serialized
/// * `S`: This is the Storage that is used during serialization
/// * `O`: This is the resulting storage type that is returned containing the serialized data
///
/// For more information about how Flavors work, please see the
/// [`flavors` module documentation](./flavors/index.html).
pub fn serialize_with_flavor<T, S, O>(value: &T, storage: S) -> Result<O>
where
    T: Serialize + ?Sized,
    S: Flavor<Output = O>,
{
    let mut serializer = Serializer {
        output: SkipStack::new(storage),
    };
    value.serialize(&mut serializer)?;
    serializer
        .output
        .into_inner()
        .finalize()
        .map_err(|_| Error::SerializeBufferFull)
}

#[cfg(feature = "heapless")]
#[cfg(test)]
mod test {
    use super::*;
    use crate::varint::{varint_max, varint_usize};
    use crate::{from_bytes, max_size::MaxSize};
    use core::fmt::Write;
    use core::ops::Deref;
    use serde::Deserialize;
    use std::collections::BTreeMap;

    #[test]
    fn ser_u8() {
        let output: Vec<u8> = to_vec(&0x05u8).unwrap();
        assert_eq!(&[5], output.deref());
    }

    #[test]
    fn ser_u16() {
        let output: Vec<u8> = to_vec(&0xA5C7u16).unwrap();
        assert_eq!(&[0xC7, 0xCB, 0x02], output.deref());
    }

    #[test]
    fn ser_u32() {
        let output: Vec<u8> = to_vec(&0xCDAB3412u32).unwrap();
        assert_eq!(&[0x92, 0xE8, 0xAC, 0xED, 0x0C], output.deref());
    }

    #[test]
    fn ser_u64() {
        let output: Vec<u8> = to_vec(&0x1234_5678_90AB_CDEFu64).unwrap();
        assert_eq!(
            &[0xEF, 0x9B, 0xAF, 0x85, 0x89, 0xCF, 0x95, 0x9A, 0x12],
            output.deref()
        );
    }

    #[test]
    fn ser_u128() {
        let output: Vec<u8> = to_vec(&0x1234_5678_90AB_CDEF_1234_5678_90AB_CDEFu128).unwrap();
        assert_eq!(
            &[
                0xEF, 0x9B, 0xAF, 0x85, 0x89, 0xCF, 0x95, 0x9A, 0x92, 0xDE, 0xB7, 0xDE, 0x8A, 0x92,
                0x9E, 0xAB, 0xB4, 0x24,
            ],
            output.deref()
        );
    }

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct BasicU8S {
        st: u16,
        ei: u8,
        ote: u128,
        sf: u64,
        tt: u32,
    }

    impl MaxSize for BasicU8S {
        const POSTCARD_MAX_SIZE: usize = {
            u16::POSTCARD_MAX_SIZE
                + u8::POSTCARD_MAX_SIZE
                + u128::POSTCARD_MAX_SIZE
                + u64::POSTCARD_MAX_SIZE
                + u32::POSTCARD_MAX_SIZE
        };
    }

    #[test]
    fn ser_struct_unsigned() {
        let input = BasicU8S {
            st: 0xABCD,
            ei: 0xFE,
            ote: 0x1234_4321_ABCD_DCBA_1234_4321_ABCD_DCBA,
            sf: 0x1234_4321_ABCD_DCBA,
            tt: 0xACAC_ACAC,
        };
        let output: Vec<u8> = to_vec(&input).unwrap();
        let deser: BasicU8S = from_bytes(&output).unwrap();

        assert_eq!(input, deser);
    }

    #[test]
    fn ser_byte_slice() {
        let input: &[u8] = &[1u8, 2, 3, 4, 5, 6, 7, 8];
        let output: Vec<u8> = to_vec(input).unwrap();
        assert_eq!(
            &[0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
            output.deref()
        );

        let mut input: Vec<u8> = Vec::new();
        for i in 0..1024 {
            input.push((i & 0xFF) as u8);
        }
        let output: Vec<u8> = to_vec(input.deref()).unwrap();
        assert_eq!(&[0x80, 0x08], &output.deref()[..2]);

        assert_eq!(output.len(), 1026);
        for (i, val) in output.deref()[2..].iter().enumerate() {
            assert_eq!((i & 0xFF) as u8, *val);
        }
    }

    #[test]
    fn ser_str() {
        let input: &str = "hello, postcard!";
        let output: Vec<u8> = to_vec(input).unwrap();
        assert_eq!(0x10, output.deref()[0]);
        assert_eq!(input.as_bytes(), &output.deref()[1..]);

        let mut input: String = String::new();
        for _ in 0..256 {
            write!(&mut input, "abcd").unwrap();
        }
        let output: Vec<u8> = to_vec(input.deref()).unwrap();
        assert_eq!(&[0x80, 0x08], &output.deref()[..2]);

        assert_eq!(output.len(), 1026);
        for ch in output.deref()[2..].chunks(4) {
            assert_eq!("abcd", core::str::from_utf8(ch).unwrap());
        }
    }

    #[test]
    fn usize_varint_encode() {
        let mut buf = [0; varint_max::<usize>()];
        let res = varint_usize(1, &mut buf);

        assert_eq!(&[1], res);

        let res = varint_usize(usize::MAX, &mut buf);

        //
        if varint_max::<usize>() == varint_max::<u32>() {
            assert_eq!(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F], res);
        } else if varint_max::<usize>() == varint_max::<u64>() {
            assert_eq!(
                &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
                res
            );
        } else {
            panic!("Update this test for 16/128 bit targets!");
        }
    }

    #[allow(dead_code)]
    #[derive(Serialize)]
    enum BasicEnum {
        Bib,
        Bim,
        Bap,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct EnumStruct {
        eight: u8,
        sixt: u16,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    enum DataEnum {
        Bib(u16),
        Bim(u64),
        Bap(u8),
        Kim(EnumStruct),
        Chi { a: u8, b: u32 },
        Sho(u16, u8),
    }

    #[test]
    fn enums() {
        let input = BasicEnum::Bim;
        let output: Vec<u8> = to_vec(&input).unwrap();
        assert_eq!(&[0x01], output.deref());

        let input = DataEnum::Bim(u64::MAX);
        let output: Vec<u8> = to_vec(&input).unwrap();
        assert_eq!(
            &[0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
            output.deref()
        );

        let input = DataEnum::Bib(u16::MAX);
        let output: Vec<u8> = to_vec(&input).unwrap();
        assert_eq!(&[0x00, 0xFF, 0xFF, 0x03], output.deref());

        let input = DataEnum::Bap(u8::MAX);
        let output: Vec<u8> = to_vec(&input).unwrap();
        assert_eq!(&[0x02, 0xFF], output.deref());

        let input = DataEnum::Kim(EnumStruct {
            eight: 0xF0,
            sixt: 0xACAC,
        });
        let output: Vec<u8> = to_vec(&input).unwrap();
        let deser: DataEnum = from_bytes(&output).unwrap();
        assert_eq!(input, deser);

        let input = DataEnum::Chi {
            a: 0x0F,
            b: 0xC7C7C7C7,
        };
        let output: Vec<u8> = to_vec(&input).unwrap();
        assert_eq!(&[0x04, 0x0F, 0xC7, 0x8F, 0x9F, 0xBE, 0x0C], output.deref());

        let input = DataEnum::Sho(0x6969, 0x07);
        let output: Vec<u8> = to_vec(&input).unwrap();
        assert_eq!(&[0x05, 0xE9, 0xD2, 0x01, 0x07], output.deref());
    }

    #[test]
    fn tuples() {
        let input = (1u8, 10u32, "Hello!");
        let output: Vec<u8> = to_vec(&input).unwrap();
        assert_eq!(
            &[1u8, 0x0A, 0x06, b'H', b'e', b'l', b'l', b'o', b'!'],
            output.deref()
        );
    }

    #[test]
    fn bytes() {
        let x: &[u8; 32] = &[0u8; 32];
        let output: Vec<u8> = to_vec(x).unwrap();
        assert_eq!(output.len(), 32);
        assert!(<[u8; 32] as MaxSize>::POSTCARD_MAX_SIZE <= output.len());

        let x: &[u8] = &[0u8; 32];
        let output: Vec<u8> = to_vec(x).unwrap();
        assert_eq!(output.len(), 33);
    }

    #[derive(Serialize)]
    pub struct NewTypeStruct(u32);

    #[derive(Serialize)]
    pub struct TupleStruct((u8, u16));

    #[test]
    fn structs() {
        let input = NewTypeStruct(5);
        let output: Vec<u8> = to_vec(&input).unwrap();
        assert_eq!(&[0x05], output.deref());

        let input = TupleStruct((0xA0, 0x1234));
        let output: Vec<u8> = to_vec(&input).unwrap();
        assert_eq!(&[0xA0, 0xB4, 0x24], output.deref());
    }

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
    struct RefStruct {
        bytes: Vec<u8>,
        str_s: String,
    }

    #[test]
    fn ref_struct() {
        let message = "hElLo";
        let bytes = [0x01, 0x10, 0x02, 0x20];
        let input = RefStruct {
            bytes: bytes.to_vec(),
            str_s: message.to_string(),
        };
        let output: Vec<u8> = to_vec(&input).unwrap();
        let deser: RefStruct = from_bytes(&output).unwrap();
        assert_eq!(input, deser);
    }

    #[test]
    fn unit() {
        let output: Vec<u8> = to_vec(&()).unwrap();
        assert_eq!(output.len(), 0);
    }

    #[test]
    fn heapless_data() {
        let mut input: Vec<u8> = Vec::new();
        input.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]);
        let output: Vec<u8> = to_vec(&input).unwrap();
        assert_eq!(&[0x04, 0x01, 0x02, 0x03, 0x04], output.deref());

        let mut input: String = String::new();
        write!(&mut input, "helLO!").unwrap();
        let output: Vec<u8> = to_vec(&input).unwrap();
        assert_eq!(&[0x06, b'h', b'e', b'l', b'L', b'O', b'!'], output.deref());

        let mut input: BTreeMap<u8, u8> = BTreeMap::new();
        input.insert(0x01, 0x05);
        input.insert(0x02, 0x06);
        input.insert(0x03, 0x07);
        input.insert(0x04, 0x08);
        let output: Vec<u8> = to_vec(&input).unwrap();
        assert_eq!(
            &[0x04, 0x01, 0x05, 0x02, 0x06, 0x03, 0x07, 0x04, 0x08],
            output.deref()
        );
    }
}
