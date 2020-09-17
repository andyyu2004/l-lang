use super::*;

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
    llvm_exec!(src);
}
