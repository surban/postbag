use postcard::{from_slice, to_vec};
use serde::{Deserialize, Serialize};

#[test]
fn changed_struct_fields() {
    #[derive(Serialize, Deserialize)]
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

    let data = to_vec(&a).unwrap();

    let b: B = from_slice(&data).unwrap();

    assert_eq!(b.f2, a.f2);
    assert_eq!(b.f4, f4_default());
}

#[test]
fn changed_struct_variant_fields() {
    #[derive(Serialize, Deserialize)]
    enum A {
        V1,
        V2 { f1: u32, f2: u32, f3: u32 },
        V3,
    }

    #[derive(Serialize, Deserialize)]
    enum B {
        V1,
        V2 {
            f2: u32,
            #[serde(default = "f4_default")]
            f4: u32,
        },
        V3,
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

    let data = to_vec(&a).unwrap();

    let b: B = from_slice(&data).unwrap();

    let B::V2 { f2, f4 } = b else {
        panic!("wrong variant")
    };
    assert_eq!(f2, a_f2);
    assert_eq!(f4, f4_default());
}
