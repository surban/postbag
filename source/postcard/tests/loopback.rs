use core::fmt::Debug;
use core::fmt::Write;
use core::ops::Deref;

use postcard::from_bytes;
use postcard::to_vec;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
struct BasicU8S {
    st: u16,
    ei: u8,
    sf: u64,
    tt: u32,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
enum BasicEnum {
    Bib,
    Bim,
    Bap,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct EnumStruct {
    eight: u8,
    sixt: u16,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
enum DataEnum {
    Bib(u16),
    Bim(u64),
    Bap(u8),
    Kim(EnumStruct),
    Chi { a: u8, b: u32 },
    Sho(u16, u8),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct NewTypeStruct(u32);

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct TupleStruct((u8, u16));

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct RefStruct<'a> {
    bytes: &'a [u8],
    str_s: &'a str,
}

#[cfg(feature = "heapless")]
#[test]
fn loopback() {
    // Basic types

    use std::collections::BTreeMap;
    test_one(());
    test_one(false);
    test_one(true);
    test_one(5u8);
    test_one(0xA5C7u16);
    test_one(0xCDAB3412u32);
    test_one(0x1234_5678_90AB_CDEFu64);

    // https://github.com/jamesmunns/postcard/pull/83
    test_one(32767i16);
    test_one(-32768i16);

    // chars
    test_one('z');
    test_one('¢');
    test_one('𐍈');
    test_one('🥺');

    // Structs
    test_one(BasicU8S {
        st: 0xABCD,
        ei: 0xFE,
        sf: 0x1234_4321_ABCD_DCBA,
        tt: 0xACAC_ACAC,
    });

    // Enums!
    test_one(BasicEnum::Bim);
    test_one(DataEnum::Bim(u64::MAX));
    test_one(DataEnum::Bib(u16::MAX));
    test_one(DataEnum::Bap(u8::MAX));
    test_one(DataEnum::Kim(EnumStruct {
        eight: 0xF0,
        sixt: 0xACAC,
    }));
    test_one(DataEnum::Chi {
        a: 0x0F,
        b: 0xC7C7C7C7,
    });
    test_one(DataEnum::Sho(0x6969, 0x07));

    // Tuples!
    test_one((0x12u8, 0xC7A5u16));

    // Structs!
    test_one(NewTypeStruct(5));
    test_one(TupleStruct((0xA0, 0x1234)));

    let mut input: Vec<u8> = Vec::new();
    input.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]);
    test_one(input);

    let mut input: String = String::new();
    write!(&mut input, "helLO!").unwrap();
    test_one(input);

    let mut input: BTreeMap<u8, u8> = BTreeMap::new();
    input.insert(0x01, 0x05);
    input.insert(0x02, 0x06);
    input.insert(0x03, 0x07);
    input.insert(0x04, 0x08);
    test_one(input);

    // `CString` (uses `serialize_bytes`/`deserialize_byte_buf`)
    #[cfg(feature = "use-std")]
    test_one(std::ffi::CString::new("heLlo").unwrap());
}

#[cfg(feature = "heapless")]
#[track_caller]
fn test_one<T>(data: T)
where
    T: Serialize + DeserializeOwned + Eq + PartialEq + Debug,
{
    let serialized: Vec<u8> = to_vec(&data).unwrap();
    let mut x: ::std::vec::Vec<u8> = vec![];
    x.extend(serialized.deref().iter().cloned());
    {
        let deserialized: T = from_bytes(&x).unwrap();
        assert_eq!(data, deserialized);
    }
}

#[cfg(feature = "use-std")]
#[test]
fn std_io_loopback() {
    use postcard::from_io;
    use postcard::to_io;

    fn test_io<T>(data: T)
    where
        T: Serialize + DeserializeOwned + Eq + PartialEq + Debug,
    {
        let serialized: ::std::vec::Vec<u8> = vec![];
        let ser = to_io(&data, serialized).unwrap();

        {
            let x = ser.clone();
            let deserialized: T = from_io(x.as_slice()).unwrap().0;
            assert_eq!(data, deserialized);
        }
    }

    test_io(DataEnum::Sho(0x6969, 0x07));
    test_io(BasicU8S {
        st: 0xABCD,
        ei: 0xFE,
        sf: 0x1234_4321_ABCD_DCBA,
        tt: 0xACAC_ACAC,
    });
}
