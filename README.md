# Postbag

[![Crates.io](https://img.shields.io/crates/v/postbag.svg)](https://crates.io/crates/postbag)
[![Documentation](https://docs.rs/postbag/badge.svg)](https://docs.rs/postbag)

Postbag is a high-performance binary [serde] codec for Rust that provides efficient data encoding with configurable levels of forward and backward compatibility.

[serde]: https://serde.rs

## Key Features

- **Full fidelity of Rust type system**: Supports all serde-compatible types including structs, enums, tuples, arrays, maps, and all primitive types
- **Efficient binary format**: Uses variable-length encoding (varint) for integers, compact representations for common types, and minimal overhead
- **Configurable compatibility**: Choose between space-efficient encoding (`Slim`) or forward/backward compatible encoding (`Full`) with field identifiers

## Quick Start

```rust
use serde::{Serialize, Deserialize};
use postbag::{to_full_vec, from_full_slice};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Person {
    name: String,
    age: u32,
}

let original = Person {
    name: "Alice".to_string(),
    age: 30,
};

// Serialize to a byte vector using Full configuration
let bytes = to_full_vec(&original).unwrap();

// Deserialize back to the original type
let deserialized: Person = from_full_slice(&bytes).unwrap();
assert_eq!(original, deserialized);
```

## Encoding Configurations

### `Full` Configuration

The `Full` configuration provides maximum compatibility and schema evolution capabilities:

- **Forward/backward compatibility**: Fields and enum variants can be reordered, added, or removed
- **Schema evolution**: Safe evolution of data structures over time
- **Numerical identifier encoding**: Fields named `_0` through `_59` are encoded with just a single byte

#### Numerical Identifier Encoding

When using `Full` configuration, fields named `_n` (where `n` is 0-59) are encoded using just a single byte instead of the full string. Use `#[serde(rename = "...")]` to specify the numerical id for each field.
This can significantly reduce serialized size for structs with many fields:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct CompactData {
    #[serde(rename = "_3")]
    my_field: u32,
    #[serde(rename = "_15")]
    another_field: String,
    // Regular field names work normally
    normal_field: bool,
}
```

This feature is entirely optional; regular field names continue to work as expected. Fields with normal and numerical names can be mixed without limitations in a single struct.

### `Slim` Configuration

The `Slim` configuration prioritizes performance and compact size:

- **Compact encoding**: Smaller serialized data size
- **Fast processing**: No string lookups during serialization/deserialization  
- **Limited schema evolution**: Fields/variants can only be added/removed at the end

**Supported changes** when using them `Slim` configuration:
- Adding fields to the end of structs (with serde defaults for deserialization)
- Removing fields from the end of structs (with serde defaults for deserialization)
- Adding enum variants at the end
- Removing enum variants from the end

**Important**: Fields and enum variants must maintain their order for compatibility when using `Slim` configuration.

## Origins

Postbag started as a fork of [postcard](https://github.com/jamesmunns/postcard) with the intent to add forward and backward compatibility to the serialized data format. While postcard provides excellent performance and compact encoding, postbag extends this foundation to support schema evolution and data format compatibility across different versions of your applications.

## License

Postbag is licensed under the [Apache 2.0 license].

[Apache 2.0 license]: https://github.com/surban/postbag/blob/master/LICENSE

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Postbag by you, shall be licensed as Apache 2.0, without any
additional terms or conditions.
