use core::fmt::Write;
use core::ops::Deref;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;

// use postcard::varint::{varint_max, varint_usize};
use postcard::{from_slice, to_vec};

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
    let deser: BasicU8S = from_slice(&output).unwrap();

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
        &[
            0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01
        ],
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
    let deser: DataEnum = from_slice(&output).unwrap();
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
    let deser: RefStruct = from_slice(&output).unwrap();
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
