use super::*;

#[test]
fn enum_nullary_ctor_vs_unit_ctor() {
    let src = r#"
    enum Option {
        Some(int),
        None,
    }

    fn main() -> int {
        Option::None();
        5
    }"#;

    expect_error!(src);
}

#[test]
fn enum_mismatching_ctor_patterns() {
    let src = r#"
    enum Option {
        Some(int),
        None,
    }

    fn main() -> int {
        let opt = Option::None;
        match opt {
            Option::Some(x) => x,
            Option::None() => 99,
        }
    }"#;

    expect_error!(src);

    let src = r#"
    enum Option {
        Some(int),
        None(),
    }

    fn main() -> int {
        let opt = Option::None();
        match opt {
            Option::Some(x) => x,
            Option::None => 99,
        }
    }"#;

    expect_error!(src);
}
