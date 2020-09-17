use super::*;

#[test]
fn enum_unit_ctor_vs_nullary_ctor() {
    let src = r#"
    enum Option {
        Some(int),
        None(),
    }

    fn main() -> int {
        Option::None;
        5
    }"#;

    expect_error!(src);
}

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
