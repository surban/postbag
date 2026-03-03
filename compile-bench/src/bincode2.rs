// Compile-time benchmark: bincode 2 (serde compat).
mod data;
mod types;

use data::*;
use types::*;

fn roundtrip<T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug>(val: &T) {
    let bytes = bincode2::serde::encode_to_vec(val, bincode2::config::standard()).unwrap();
    let (decoded, _): (T, _) =
        bincode2::serde::decode_from_slice(&bytes, bincode2::config::standard()).unwrap();
    assert_eq!(val, &decoded);
}

fn main() {
    roundtrip(&simple());
    roundtrip(&Color::Red);
    roundtrip(&address());
    roundtrip(&person());
    roundtrip(&company());
    roundtrip(&Message::Text("t".into()));
    roundtrip(&config());
    roundtrip(&config_entry());
    roundtrip(&ConfigValue::Bool(true));
    roundtrip(&matrix());
    roundtrip(&time_series());
    roundtrip(&document());
    roundtrip(&event());
    roundtrip(&api_response());
    roundtrip(&database_record());
    roundtrip(&workspace());
    println!("bincode2 roundtrip OK");
}
