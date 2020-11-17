use super::*;

#[test]
fn llvm_immediate_loop_break() {
    let src = r#"
    fn main() -> int {
        loop {
            break
        };
        0
    }"#;

    assert_eq!(llvm_exec!(src), 0);
}

#[test]
fn llvm_loop_break() {
    let src = r#"
    fn main() -> int {
        let mut x = 0;
        loop {
            if x > 5 {
                break
            };
            x = x + 1;
        };
        x
    }"#;

    assert_eq!(llvm_exec!(src), 6);
}
