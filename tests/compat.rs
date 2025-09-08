use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt::Debug;

use postbag::{
    cfg::{Cfg, Full, Slim},
    deserialize, serialize,
};

/// Transform from one type to another via serialization followed by deserialization.
#[track_caller]
pub fn transform<T, R, CFG>(value: &T) -> R
where
    T: Serialize + DeserializeOwned + Debug + Eq,
    R: DeserializeOwned,
    CFG: Cfg,
{
    let mut serialized = Vec::new();
    serialize::<CFG, _, _>(&value, &mut serialized).expect("serialization failed");
    println!("{serialized:02x?}");
    dbg!(serialized.len());

    let deserialized: T = deserialize::<CFG, _, _>(serialized.as_slice()).expect("deserialization failed");

    assert_eq!(*value, deserialized, "deserialized value does not match original value");

    deserialize::<CFG, _, _>(serialized.as_slice()).expect("deserialization to transformed type failed")
}

#[test]
fn changed_struct_fields() {
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct A {
        f1: u32,
        f2: u32,
        f3: u32,
    }

    #[derive(Serialize, Deserialize)]
    struct B {
        f2: u32,
        #[serde(default = "f4_default")]
        f4: u32,
    }

    const fn f4_default() -> u32 {
        4
    }

    let a = A { f1: 1, f2: 2, f3: 3 };

    let b: B = transform::<_, _, Full>(&a);

    assert_eq!(b.f2, a.f2);
    assert_eq!(b.f4, f4_default());
}

#[test]
fn added_struct_fields() {
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct A {
        f1: u32,
        f2: u32,
        f3: u32,
    }

    #[derive(Serialize, Deserialize)]
    struct B {
        f1: u32,
        f2: u32,
        f3: u32,
        #[serde(default = "f4_default")]
        f4: u32,
    }

    const fn f4_default() -> u32 {
        4
    }

    let a = A { f1: 1, f2: 2, f3: 3 };

    let b: B = transform::<_, _, Full>(&a);
    assert_eq!(b.f1, a.f1);
    assert_eq!(b.f2, a.f2);
    assert_eq!(b.f3, a.f3);
    assert_eq!(b.f4, f4_default());

    let b: B = transform::<_, _, Slim>(&a);
    assert_eq!(b.f1, a.f1);
    assert_eq!(b.f2, a.f2);
    assert_eq!(b.f3, a.f3);
    assert_eq!(b.f4, f4_default());
}

#[test]
fn changed_struct_variant_fields() {
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    enum A {
        V1,
        V2 { f1: u32, f2: u32, f3: u32 },
        V3,
    }

    #[derive(Serialize, Deserialize)]
    enum B {
        V1a,
        V3b,
        V2 {
            f2: u32,
            #[serde(default = "f4_default")]
            f4: u32,
        },
    }

    const fn f4_default() -> u32 {
        4
    }

    let a_f2 = 2;
    let a = A::V2 { f1: 1, f2: a_f2, f3: 3 };

    let b: B = transform::<_, _, Full>(&a);

    let B::V2 { f2, f4 } = b else { panic!("wrong variant") };
    assert_eq!(f2, a_f2);
    assert_eq!(f4, f4_default());
}

#[test]
fn added_struct_variant_fields() {
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    enum A {
        V1,
        V2 { f1: u32, f2: u32, f3: u32 },
        V3,
    }

    #[derive(Serialize, Deserialize)]
    enum B {
        V1a,
        V2 {
            f1: u32,
            f2: u32,
            f3: u32,
            #[serde(default = "f4_default")]
            f4: u32,
        },
    }

    const fn f4_default() -> u32 {
        4
    }

    let a_f1 = 1;
    let a_f2 = 2;
    let a_f3 = 3;
    let a = A::V2 { f1: a_f1, f2: a_f2, f3: a_f3 };

    let b: B = transform::<_, _, Full>(&a);
    let B::V2 { f1, f2, f3, f4 } = b else { panic!("wrong variant") };
    assert_eq!(f1, a_f1);
    assert_eq!(f2, a_f2);
    assert_eq!(f3, a_f3);
    assert_eq!(f4, f4_default());

    let b: B = transform::<_, _, Slim>(&a);
    let B::V2 { f1, f2, f3, f4 } = b else { panic!("wrong variant") };
    assert_eq!(f1, a_f1);
    assert_eq!(f2, a_f2);
    assert_eq!(f3, a_f3);
    assert_eq!(f4, f4_default());
}

#[test]
fn removed_struct_fields_nested_struct() {
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct A {
        f1: u32,
        f2: u32,
        f3: u32,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct XA {
        a: A,
        x: u32,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct B {
        f1: u32,
        f2: u32,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct XB {
        a: B,
        x: u32,
    }

    let xa = XA { a: A { f1: 1, f2: 2, f3: 3 }, x: 99 };

    let xb: XB = transform::<_, _, Full>(&xa);
    assert_eq!(xb.a.f1, xa.a.f1);
    assert_eq!(xb.a.f2, xa.a.f2);
    assert_eq!(xb.x, xa.x);

    let xb: XB = transform::<_, _, Slim>(&xa);
    assert_eq!(xb.a.f1, xa.a.f1);
    assert_eq!(xb.a.f2, xa.a.f2);
    assert_eq!(xb.x, xa.x);
}

#[test]
fn removed_struct_fields_nested_tuple() {
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct A {
        f1: u32,
        f2: u32,
        f3: u32,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct B {
        f1: u32,
        f2: u32,
    }

    let xa = (A { f1: 1, f2: 2, f3: 3 }, 99);

    let xb: (B, u32) = transform::<_, _, Full>(&xa);
    assert_eq!(xb.0.f1, xa.0.f1);
    assert_eq!(xb.0.f2, xa.0.f2);
    assert_eq!(xb.1, xa.1);

    let xb: (B, u32) = transform::<_, _, Slim>(&xa);
    assert_eq!(xb.0.f1, xa.0.f1);
    assert_eq!(xb.0.f2, xa.0.f2);
    assert_eq!(xb.1, xa.1);
}

#[test]
fn added_enum_variants_slim_encoding() {
    // Original enum with 3 variants
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    enum Original {
        Variant1,
        Variant2(u32),
        Variant3 { value: String },
    }

    // Extended enum with additional variants at the end
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    enum Extended {
        Variant1,
        Variant2(u32),
        Variant3 {
            value: String,
        },
        Variant4,
        Variant5(bool),
        #[serde(other)]
        Unknown,
    }

    // Even more extended enum for backward compatibility testing
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    enum MoreExtended {
        Variant1,
        Variant2(u32),
        Variant3 {
            value: String,
        },
        Variant4,
        Variant5(bool),
        Variant6 {
            x: i32,
            y: i32,
        },
        #[serde(other)]
        Unknown,
    }

    // Test forward compatibility: Original -> Extended
    let original_v1 = Original::Variant1;
    let extended_v1: Extended = transform::<_, _, Slim>(&original_v1);
    assert_eq!(extended_v1, Extended::Variant1);

    let original_v2 = Original::Variant2(42);
    let extended_v2: Extended = transform::<_, _, Slim>(&original_v2);
    assert_eq!(extended_v2, Extended::Variant2(42));

    let original_v3 = Original::Variant3 { value: "test".to_string() };
    let extended_v3: Extended = transform::<_, _, Slim>(&original_v3);
    assert_eq!(extended_v3, Extended::Variant3 { value: "test".to_string() });

    // Test backward compatibility: Extended -> Original (with #[serde(other)])
    let extended_v4 = Extended::Variant4;
    let mut serialized = Vec::new();
    serialize::<Slim, _, _>(&extended_v4, &mut serialized).expect("serialization failed");

    // This should deserialize to Unknown variant when using Original enum with #[serde(other)]
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    enum OriginalWithOther {
        Variant1,
        Variant2(u32),
        Variant3 {
            value: String,
        },
        #[serde(other)]
        Unknown,
    }

    let deserialized: OriginalWithOther =
        deserialize::<Slim, _, _>(serialized.as_slice()).expect("deserialization failed");
    assert_eq!(deserialized, OriginalWithOther::Unknown);

    let extended_v5 = Extended::Variant5(true);
    let mut serialized = Vec::new();
    serialize::<Slim, _, _>(&extended_v5, &mut serialized).expect("serialization failed");
    let deserialized: OriginalWithOther =
        deserialize::<Slim, _, _>(serialized.as_slice()).expect("deserialization failed");
    assert_eq!(deserialized, OriginalWithOther::Unknown);

    // Test compatibility with even more extended version
    let more_extended_v6 = MoreExtended::Variant6 { x: 10, y: 20 };
    let mut serialized = Vec::new();
    serialize::<Slim, _, _>(&more_extended_v6, &mut serialized).expect("serialization failed");

    // Should deserialize to Unknown in Extended enum
    let deserialized: Extended =
        deserialize::<Slim, _, _>(serialized.as_slice()).expect("deserialization failed");
    assert_eq!(deserialized, Extended::Unknown);

    // Should also deserialize to Unknown in OriginalWithOther enum
    let deserialized: OriginalWithOther =
        deserialize::<Slim, _, _>(serialized.as_slice()).expect("deserialization failed");
    assert_eq!(deserialized, OriginalWithOther::Unknown);

    // Test that existing variants still work across all versions
    let more_extended_v1 = MoreExtended::Variant1;
    let extended_v1: Extended = transform::<_, _, Slim>(&more_extended_v1);
    assert_eq!(extended_v1, Extended::Variant1);

    let original_v1: OriginalWithOther = transform::<_, _, Slim>(&more_extended_v1);
    assert_eq!(original_v1, OriginalWithOther::Variant1);
}

#[test]
fn added_enum_variants_full_encoding() {
    // Original enum with 3 variants
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    enum Original {
        Variant1,
        Variant2(u32),
        Variant3 { value: String },
    }

    // Extended enum with additional variants at the end
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    enum Extended {
        Variant1,
        Variant2(u32),
        Variant3 {
            value: String,
        },
        Variant4,
        Variant5(bool),
        #[serde(other)]
        Unknown,
    }

    // Even more extended enum for backward compatibility testing
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    enum MoreExtended {
        Variant1,
        Variant2(u32),
        Variant3 {
            value: String,
        },
        Variant4,
        Variant5(bool),
        Variant6 {
            x: i32,
            y: i32,
        },
        #[serde(other)]
        Unknown,
    }

    // Test forward compatibility: Original -> Extended
    let original_v1 = Original::Variant1;
    let extended_v1: Extended = transform::<_, _, Full>(&original_v1);
    assert_eq!(extended_v1, Extended::Variant1);

    let original_v2 = Original::Variant2(42);
    let extended_v2: Extended = transform::<_, _, Full>(&original_v2);
    assert_eq!(extended_v2, Extended::Variant2(42));

    let original_v3 = Original::Variant3 { value: "test".to_string() };
    let extended_v3: Extended = transform::<_, _, Full>(&original_v3);
    assert_eq!(extended_v3, Extended::Variant3 { value: "test".to_string() });

    // Test backward compatibility: Extended -> Original (with #[serde(other)])
    let extended_v4 = Extended::Variant4;
    let mut serialized = Vec::new();
    serialize::<Full, _, _>(&extended_v4, &mut serialized).expect("serialization failed");

    // This should deserialize to Unknown variant when using Original enum with #[serde(other)]
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    enum OriginalWithOther {
        Variant1,
        Variant2(u32),
        Variant3 {
            value: String,
        },
        #[serde(other)]
        Unknown,
    }

    let deserialized: OriginalWithOther =
        deserialize::<Full, _, _>(serialized.as_slice()).expect("deserialization failed");
    assert_eq!(deserialized, OriginalWithOther::Unknown);

    let extended_v5 = Extended::Variant5(true);
    let mut serialized = Vec::new();
    serialize::<Full, _, _>(&extended_v5, &mut serialized).expect("serialization failed");
    let deserialized: OriginalWithOther =
        deserialize::<Full, _, _>(serialized.as_slice()).expect("deserialization failed");
    assert_eq!(deserialized, OriginalWithOther::Unknown);

    // Test compatibility with even more extended version
    let more_extended_v6 = MoreExtended::Variant6 { x: 10, y: 20 };
    let mut serialized = Vec::new();
    serialize::<Full, _, _>(&more_extended_v6, &mut serialized).expect("serialization failed");

    // Should deserialize to Unknown in Extended enum
    let deserialized: Extended =
        deserialize::<Full, _, _>(serialized.as_slice()).expect("deserialization failed");
    assert_eq!(deserialized, Extended::Unknown);

    // Should also deserialize to Unknown in OriginalWithOther enum
    let deserialized: OriginalWithOther =
        deserialize::<Full, _, _>(serialized.as_slice()).expect("deserialization failed");
    assert_eq!(deserialized, OriginalWithOther::Unknown);

    // Test that existing variants still work across all versions
    let more_extended_v1 = MoreExtended::Variant1;
    let extended_v1: Extended = transform::<_, _, Full>(&more_extended_v1);
    assert_eq!(extended_v1, Extended::Variant1);

    let original_v1: OriginalWithOther = transform::<_, _, Full>(&more_extended_v1);
    assert_eq!(original_v1, OriginalWithOther::Variant1);
}
