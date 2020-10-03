use super::*;

#[test]
fn test_simple_deref() {
    typeck_expr!("*(box 5)");
}

// #[test]
fn test_access_field_of_boxed_struct() {
    let src = r#"
    struct S { x: int };

    fn main() -> int {
        let s = box S { x: 5 };
        s.x
    }
    "#;
    typeck!(src);
}
