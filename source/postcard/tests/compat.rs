use postcard::{from_bytes, to_vec};
use serde::{Deserialize, Serialize};

#[test]
fn changed_fields() {
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

    let b: B = from_bytes(&data).unwrap();

    assert_eq!(b.f2, a.f2);
    assert_eq!(b.f4, f4_default());
}

// So what do we actually want to do?
// Do the identifiers? No, probably this will add
// So, we just accept the identifiers?
// Yes, also it might be a reason to actually keep Postcard as a codec.
// Or we make this thingy here configurable?
