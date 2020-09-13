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
fn test_box_deref_assign() {
    let src = r#"
    fn main() -> int {
        let x = box 5;
        x;
        5
    }"#;
    llvm_exec!(src);
}
