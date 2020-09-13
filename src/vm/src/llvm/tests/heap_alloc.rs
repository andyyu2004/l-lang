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
