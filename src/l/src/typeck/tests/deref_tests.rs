use super::*;

#[test]
fn test_simple_deref() {
    typeck_expr!("*(box 5)");
}

#[test]
fn struct_field_box_autoderef() {
    let src = r#"
    struct S { x: int };

    fn main() -> int {
        let s = box box box box S { x: 5 };
        s.x
    }
    "#;
    typeck!(src);
}
