use super::*;

#[test]
fn llvm_match_simple_enum() {
    let src = r#"
    enum E {
       A, B
    }

    fn main() -> int {
        match E::A {
            E::A => 555,
            E::B => 999,
        }
    }"#;

    assert_eq!(llvm_jit!(src), 555);

    let src = r#"
    enum E {
       A, B
    }

    fn main() -> int {
        match E::B {
            E::A => 555,
            E::B => 999,
        }
    }"#;

    assert_eq!(llvm_jit!(src), 999);
}

#[test]
fn llvm_construct_enums() {
    let src = r#"
    enum Option {
        Some(int),
        None,
    }

    enum WeirdEither {
        Left { left: int },
        Right(bool),
    }

    fn main() -> int {
        Option::Some(5);
        // Option::None;
        // WeirdEither::Left { left: 8 };
        // WeirdEither::Right(false);
        0
    }"#;

    // if it doesn't crash its a pass :)
    llvm_jit!(src);
}
