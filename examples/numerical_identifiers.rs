use postbag::{Full, deserialize, serialize};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct RegularData {
    my_field: u32,
    another_field: String,
    third_field: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct CompactData {
    #[serde(rename = "_3")]
    my_field: u32,
    #[serde(rename = "_15")]
    another_field: String,
    #[serde(rename = "_42")]
    third_field: bool,
}

fn main() {
    let data_regular = RegularData { my_field: 42, another_field: "hello".to_string(), third_field: true };

    let data_compact = CompactData { my_field: 42, another_field: "hello".to_string(), third_field: true };

    // Serialize both versions
    let mut regular_buffer = Vec::new();
    serialize::<Full, _, _>(&data_regular, &mut regular_buffer).unwrap();

    let mut compact_buffer = Vec::new();
    serialize::<Full, _, _>(&data_compact, &mut compact_buffer).unwrap();

    println!("Regular field names: {} bytes", regular_buffer.len());
    println!("Compact field names: {} bytes", compact_buffer.len());
    println!("Space saved: {} bytes", regular_buffer.len() - compact_buffer.len());

    // Verify both can be deserialized correctly
    let regular_deserialized: RegularData = deserialize::<Full, _, _>(regular_buffer.as_slice()).unwrap();
    let compact_deserialized: CompactData = deserialize::<Full, _, _>(compact_buffer.as_slice()).unwrap();

    assert_eq!(data_regular, regular_deserialized);
    assert_eq!(data_compact, compact_deserialized);

    println!("✓ Both versions serialize and deserialize correctly!");
}
