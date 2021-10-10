use super::*;

#[test]
fn test_box_alloc() {
    let src = r#"
    fn main() -> int {
        let x = box 5;
        x;
        5
    }"#;
    llvm_exec!(src);
}

#[test]
fn test_box_deref() {
    let src = r#"
    fn main() -> int {
        let x = box 5;
        *x
    }"#;
    llvm_exec!(src);
}

#[test]
fn struct_field_box_autoderef() {
    let src = r#"
    struct S { x: int };

    fn main() -> int {
        let s = box box box box box box box S { x: 5 };
        s.x
    }
    "#;
    assert_eq!(llvm_exec!(src), 5);
}

#[test]
fn test_box_deref_assign() {
    let src = r#"
    fn main() -> int {
        let ptr = box 5;
        mutate(ptr);
        *ptr
    }

    fn mutate(ptr: &int) {
        *ptr = 99;
    }
    "#;

    assert_eq!(llvm_exec!(src), 99);
}

#[test]
fn test_double_pointers() {
    let src = r#"
    fn main() -> int {
        **double_ptr()
    }

    fn double_ptr() -> &&int {
        box box 5
    }"#;

    assert_eq!(llvm_exec!(src), 5);
}

#[test]
fn multi_deref_across_functions() {
    let src = r#"
    fn main() -> int {
        let ptr = f1();
        ***ptr
    }

    fn f1() -> &&&int {
        box f2()
    }

    fn f2() -> &&int {
        box f3()
    }

    fn f3() -> &int {
        box 20
    }"#;

    assert_eq!(llvm_exec!(src), 20);
}
