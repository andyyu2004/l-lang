use super::*;

#[test]
fn llvm_tuple_field_access() {
    let out = llvm_exec!("fn main() -> int { (1,8).1 }");
    assert_eq!(out, 8);
}

#[test]
fn llvm_singleton_struct_construction() {
    let src = r#"
    struct S { x: int }

    fn main() -> int {
        S { x: 5 }.x
    }
    "#;
    assert_eq!(llvm_exec!(src), 5)
}

#[test]
fn llvm_tuple_in_struct() {
    let src = r#"
    struct S {
        x: int,
        y: (int, bool, int),
    }

    fn main() -> int {
        S { x: 5, y: (4, false, 9) }.y.2
    }
    "#;
    assert_eq!(llvm_exec!(src), 9)
}
