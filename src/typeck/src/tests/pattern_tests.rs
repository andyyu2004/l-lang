use super::*;

#[test]
fn typeck_struct_pattern_field_bound_more_than_once() {
    let src = r#"
    struct S {
        b: bool,
        x: int,
        y: int,
    }

    fn main() -> int {
        let s = S {
            x: 9,
            b: false,
            y: 7,
        };
        let S { x, y, x: i } = s;
        x - y
    }
    "#;

    expect_error!(src);
}

#[test]
fn typeck_invalid_struct_pattern_unknown_field() {
    let src = r#"
    struct S {}

    fn main() -> int {
        let s = S { };
        let S { x } = s;
        0
    }
    "#;

    expect_error!(src);
}

#[test]
fn typeck_empty_struct_pattern() {
    let src = r#"
    struct S {}

    fn main() -> int {
        let s = S {};
        let S {} = s;
        0
    }
    "#;

    typeck!(src);
}
