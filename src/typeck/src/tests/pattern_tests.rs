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
