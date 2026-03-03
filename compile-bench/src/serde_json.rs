// Compile-time benchmark: serde_json.
mod data;
mod types;

use data::*;
use types::*;

fn roundtrip<T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug>(val: &T) {
    let bytes = serde_json::to_vec(val).unwrap();
    let decoded: T = serde_json::from_slice(&bytes).unwrap();
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
    println!("serde_json roundtrip OK");
}
