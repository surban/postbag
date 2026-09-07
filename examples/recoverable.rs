use postbag::{from_full_slice, recoverable::Recoverable, to_full_vec};
use serde::{Deserialize, Serialize};

// Version 1 of the data model, as used by the sender.
#[derive(Serialize)]
enum ThemeV1 {
    Solarized,
}

#[derive(Serialize)]
struct SettingsV1 {
    theme: ThemeV1,
    refresh_seconds: u32,
}

#[derive(Serialize)]
struct MessageV1 {
    sequence: u32,
    settings: SettingsV1,
    note: String,
}

// Version 2 no longer recognizes the Solarized variant. Light is the fallback
// used when the enclosing SettingsV2 value is recovered through Default.
#[derive(Debug, Default, Deserialize)]
enum ThemeV2 {
    #[default]
    Light,
    Dark,
}

#[derive(Debug, Default, Deserialize)]
struct SettingsV2 {
    theme: ThemeV2,
    refresh_seconds: u32,
}

// Deserializing SettingsV2 directly lets its error abort the entire message.
#[derive(Deserialize)]
struct StrictMessageV2 {
    sequence: u32,
    settings: SettingsV2,
    note: String,
}

// Isolating settings in Recoverable confines a decoding failure to that field.
#[derive(Debug, Deserialize)]
struct RecoveringMessageV2 {
    sequence: u32,
    note: String,
    settings: Recoverable<SettingsV2>,
}

fn main() {
    // Encode a message using the old schema.
    let message = MessageV1 {
        sequence: 42,
        settings: SettingsV1 { theme: ThemeV1::Solarized, refresh_seconds: 30 },
        note: "This field follows the incompatible one".into(),
    };
    let bytes = to_full_vec(&message).unwrap();

    // The unknown theme prevents a strict V2 reader from reaching the remaining fields.
    let strict_error = match from_full_slice::<StrictMessageV2>(&bytes) {
        Err(err) => err,
        Ok(message) => panic!(
            "unexpectedly decoded sequence {}, settings {:?}, note {:?}",
            message.sequence, message.settings, message.note
        ),
    };
    println!("Without recovery, the whole message fails: {strict_error}");

    // Recoverable consumes the incompatible settings value in isolation, replaces it
    // with SettingsV2::default(), and then continues decoding the following note.
    let recovered: RecoveringMessageV2 = from_full_slice(&bytes).unwrap();
    println!("\nWith Recoverable<SettingsV2>:");
    println!("  sequence: {}", recovered.sequence);
    println!("  theme: {:?}", recovered.settings.theme);
    println!("  refresh seconds: {}", recovered.settings.refresh_seconds);
    println!("  settings recovered: {}", Recoverable::is_recovered(&recovered.settings));
    println!("  note: {}", recovered.note);

    assert_eq!(recovered.sequence, 42);
    assert!(Recoverable::is_recovered(&recovered.settings));
    assert_eq!(recovered.note, "This field follows the incompatible one");
}
