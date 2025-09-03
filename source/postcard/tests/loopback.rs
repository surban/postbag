use postcard::{Error, from_slice, to_vec};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::DeserializeOwned};
use std::fmt::Debug;
use std::fmt::Write;
use std::{collections::BTreeMap, io::ErrorKind};

/// Performs serialization followed by deserialization and checks that the
/// deserialized value is unchanged.
#[track_caller]
pub fn loopback<T>(value: T)
where
    T: Serialize + DeserializeOwned + Debug + Eq,
{
    let serialized = to_vec(&value).expect("serialization failed");
    dbg!(serialized.len());

    let deserialized: T = from_slice(&serialized).expect("deserialization failed");

    assert_eq!(
        value, deserialized,
        "deserialized value does not match original value"
    );
}

// =============================================================================
// Primitive Types Tests
// =============================================================================

#[test]
fn primitives_unsigned_integers() {
    loopback(0x05u8);
    loopback(0xA5C7u16);
    loopback(0xCDAB3412u32);
    loopback(0x1234_5678_90AB_CDEFu64);
    loopback(0x1234_5678_90AB_CDEF_1234_5678_90AB_CDEFu128);
}

#[test]
fn primitives_signed_integers() {
    loopback(32767i16); // positive edge
    loopback(-32768i16); // negative edge
    loopback(-19490127978232325886905073712831_i128); // large negative i128
}

#[test]
fn primitives_booleans() {
    loopback(false);
    loopback(true);
}

#[test]
fn primitives_characters() {
    loopback('a'); // simple char
    loopback('z'); // ASCII char
    loopback('¢'); // Latin char
    loopback('𐍈'); // Gothic char
    loopback('🥺'); // Emoji char
}

#[test]
fn primitives_unit_type() {
    loopback(());
}

// =============================================================================
// Struct and Newtype Tests
// =============================================================================

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
struct BasicU8S {
    st: u16,
    ei: u8,
    ote: u128,
    sf: u64,
    tt: u32,
}

#[test]
fn structs_basic() {
    let data = BasicU8S {
        st: 0xABCD,
        ei: 0xFE,
        ote: 0x1234_4321_ABCD_DCBA_1234_4321_ABCD_DCBA,
        sf: 0x1234_4321_ABCD_DCBA,
        tt: 0xACAC_ACAC,
    };
    loopback(data);
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct NewTypeStruct(u32);

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct TupleStruct((u8, u16));

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct DualTupleStruct(u8, u16);

#[test]
fn structs_variants() {
    loopback(NewTypeStruct(5));
    loopback(TupleStruct((0xA0, 0x1234)));
    loopback(DualTupleStruct(0xA0, 0x1234));
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct RefStruct {
    bytes: std::vec::Vec<u8>,
    str_s: std::string::String,
}

#[test]
fn structs_with_references() {
    let message = "hElLo";
    let bytes = [0x01, 0x10, 0x02, 0x20];
    loopback(RefStruct {
        bytes: bytes.to_vec(),
        str_s: message.to_string(),
    });
}

// =============================================================================
// Enum Tests
// =============================================================================

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

#[test]
fn enums_basic() {
    loopback(BasicEnum::Bim);
}

#[test]
fn enums_with_data() {
    loopback(DataEnum::Bim(u64::MAX));
    loopback(DataEnum::Bib(u16::MAX));
    loopback(DataEnum::Bap(u8::MAX));
    loopback(DataEnum::Kim(EnumStruct {
        eight: 0xF0,
        sixt: 0xACAC,
    }));
    loopback(DataEnum::Chi {
        a: 0x0F,
        b: 0xC7C7C7C7,
    });
    loopback(DataEnum::Sho(0x6969, 0x07));
}

// =============================================================================
// Collection Tests
// =============================================================================

#[test]
fn collections_strings() {
    let input: &str = "hello, postcard!";
    loopback(input.to_string());

    let mut input: String = String::new();
    for _ in 0..256 {
        write!(&mut input, "abcd").unwrap();
    }
    loopback(input);

    // Additional string test
    let mut input: String = String::new();
    write!(&mut input, "helLO!").unwrap();
    loopback(input);
}

#[test]
fn collections_byte_slices_and_vecs() {
    let input: &[u8] = &[1u8, 2, 3, 4, 5, 6, 7, 8];
    loopback(input.to_vec());

    let mut input: Vec<u8> = Vec::new();
    for i in 0..1024 {
        input.push((i & 0xFF) as u8);
    }
    loopback(input);

    // Additional vec tests
    let mut input: Vec<u8> = Vec::new();
    input.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]);
    loopback(input);

    let input = vec![255, 255, 255, 0, 0, 0, 0, 0];
    loopback(input);
}

#[test]
fn collections_maps() {
    let mut input: BTreeMap<u8, u8> = BTreeMap::new();
    input.insert(0x01, 0x05);
    input.insert(0x02, 0x06);
    input.insert(0x03, 0x07);
    input.insert(0x04, 0x08);
    loopback(input);
}

#[test]
fn collections_cstring() {
    // CString (uses serialize_bytes/deserialize_byte_buf)
    loopback(std::ffi::CString::new("heLlo").unwrap());
}

// =============================================================================
// Tuple Tests
// =============================================================================

#[test]
fn tuples_basic() {
    loopback((1u8, 10u32, "Hello!".to_string()));
    loopback((0x12u8, 0xC7A5u16));
}

// =============================================================================
// Bytes and Special Serialization Tests
// =============================================================================

#[derive(Debug, Eq, PartialEq)]
pub struct ByteSliceStruct {
    bytes: std::vec::Vec<u8>,
}

impl Serialize for ByteSliceStruct {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialization is generic for all slice types, so the default serialization of byte
        // slices does not use `Serializer::serialize_bytes`.
        serializer.serialize_bytes(&self.bytes)
    }
}

impl<'de> Deserialize<'de> for ByteSliceStruct {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialization of byte slices is specialized for byte slices, so the default
        // deserialization will call `Deserializer::deserialize_bytes`.
        Ok(Self {
            bytes: Deserialize::deserialize(deserializer)?,
        })
    }
}

#[test]
fn bytes_arrays_and_custom_serialization() {
    loopback([0u8; 32]);
    loopback(vec![0u8; 32]);
    loopback(ByteSliceStruct {
        bytes: vec![0u8; 32],
    });
}

// =============================================================================
// Zero-Sized Type Tests
// =============================================================================

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ZSTStruct;

#[test]
fn zero_sized_types() {
    let input = vec![ZSTStruct, ZSTStruct, ZSTStruct];
    loopback(input);

    let input = vec![ZSTStruct, ZSTStruct, ZSTStruct, ZSTStruct];
    loopback(input);
}

// =============================================================================
// Error Handling and Edge Case Tests
// =============================================================================

#[test]
fn error_handling_vec_bounds() {
    // This won't actually prove anything since tests will likely always be
    // run on devices with larger amounts of memory, but it can't hurt.
    assert!(matches!(
        from_slice::<Vec<u8>>(&[(1 << 7) | 8, 255, 255, 255, 0, 0, 0, 0, 0]),
        Err(Error::Io(io)) if io.kind() == ErrorKind::UnexpectedEof
    ));
}

#[test]
fn varint_boundary_tests() {
    loopback(u32::MAX);

    let deser: postcard::Result<u32> = from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0x1F]);
    assert!(matches!(deser, Err(Error::DeserializeBadVarint)));
}
