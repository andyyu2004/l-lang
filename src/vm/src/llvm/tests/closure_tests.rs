use super::*;

#[test]
fn llvm_simple_closure() {
    let src = r#"
    fn main() -> int {
        let x = 5;
        let f = fn() => x;
        return f();
    }"#;

    llvm_exec!(src);
}
