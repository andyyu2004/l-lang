use super::*;

// tuple tests are also here as they are similar to structs in many aspects
#[test]
fn check_tuple_out_of_bounds() {
    expect_type_error!("fn main() -> int { (1,2).2 }")
}

#[test]
fn check_struct_duplicate_fields() {
    let src = r#"struct S { x:int, x: int }"#;
    expect_type_error!(src);
}

#[test]
fn check_struct_field_assign() {
    let src = r#"
    struct S { x: int }

    fn main() -> int {
        let mut s = S { x: 4 };
        s.x = 5;
        0
    }"#;
    typeck!(src);
}

#[test]
fn check_struct_badly_typed() {
    expect_type_error!("struct S { x: int } fn main() -> int { S { x: false }; 5 }");
}

#[test]
fn check_struct_missing_fields() {
    expect_type_error!("struct S { x: int, y: bool } fn main() -> int { S { x: 3 }; 5 }");
}

#[test]
fn check_struct_set_field_multiple() {
    expect_type_error!("struct S { x: int } fn main() -> int { S { x: 4, x: 5 }; 5 }");
}

#[test]
fn check_struct_with_type_annotation() {
    let src = "struct S { x: int } fn main() -> int { let s: S = S { x: 4 }; s.x }";
    typeck!(src);
}

#[test]
fn check_struct_differing_orders() {
    let src = r#"
    struct S { x: int, y: bool }

    fn main() -> int {
        S {
            y: false,
            x: 4,
        };
        5
    }
    "#;
    typeck!(src);
}
