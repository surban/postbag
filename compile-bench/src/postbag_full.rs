// Compile-time benchmark: postbag Full mode.
mod data;
mod types;

use data::*;
use types::*;

fn roundtrip<T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug>(val: &T) {
    let bytes = postbag::to_full_vec(val).unwrap();
    let decoded: T = postbag::from_full_slice(&bytes).unwrap();
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
    println!("postbag Full roundtrip OK");
}
