use postcard::{Cfg, Config, from_slice_with_cfg, to_vec_with_cfg};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt::Debug;

type WithIdents = Config<true>;
type WithoutIdents = Config<false>;

/// Transform from one type to another via serialization followed by deserialization.
#[track_caller]
pub fn transform<T, R, CFG>(value: &T) -> R
where
    T: Serialize + DeserializeOwned + Debug + Eq,
    R: DeserializeOwned,
    CFG: Cfg,
{
    let serialized = to_vec_with_cfg::<_, CFG>(&value).expect("serialization failed");
    println!("{serialized:02x?}");
    dbg!(serialized.len());

    let deserialized: T =
        from_slice_with_cfg::<_, CFG>(&serialized).expect("deserialization failed");

    assert_eq!(
        *value, deserialized,
        "deserialized value does not match original value"
    );

    from_slice_with_cfg::<_, CFG>(&serialized).expect("deserialization to transformed type failed")
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

    let a = A {
        f1: 1,
        f2: 2,
        f3: 3,
    };

    let b: B = transform::<_, _, WithIdents>(&a);

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

    let a = A {
        f1: 1,
        f2: 2,
        f3: 3,
    };

    let b: B = transform::<_, _, WithIdents>(&a);
    assert_eq!(b.f1, a.f1);
    assert_eq!(b.f2, a.f2);
    assert_eq!(b.f3, a.f3);
    assert_eq!(b.f4, f4_default());

    let b: B = transform::<_, _, WithoutIdents>(&a);
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
    let a = A::V2 {
        f1: 1,
        f2: a_f2,
        f3: 3,
    };

    let b: B = transform::<_, _, WithIdents>(&a);

    let B::V2 { f2, f4 } = b else {
        panic!("wrong variant")
    };
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
    let a = A::V2 {
        f1: a_f1,
        f2: a_f2,
        f3: a_f3,
    };

    let b: B = transform::<_, _, WithIdents>(&a);
    let B::V2 { f1, f2, f3, f4 } = b else {
        panic!("wrong variant")
    };
    assert_eq!(f1, a_f1);
    assert_eq!(f2, a_f2);
    assert_eq!(f3, a_f3);
    assert_eq!(f4, f4_default());

    let b: B = transform::<_, _, WithoutIdents>(&a);
    let B::V2 { f1, f2, f3, f4 } = b else {
        panic!("wrong variant")
    };
    assert_eq!(f1, a_f1);
    assert_eq!(f2, a_f2);
    assert_eq!(f3, a_f3);
    assert_eq!(f4, f4_default());
}
