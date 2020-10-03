use super::*;

#[test]
fn simple_conditionals() {
    let src = "fn main() -> int { if true { 20 } else { 30 } }";
    assert_eq!(llvm_exec!(src), 20);

    let src = "fn main() -> int { if false { 20 } else { 30 } }";
    assert_eq!(llvm_exec!(src), 30);
}

#[test]
fn simple_literal_match() {
    let src = r#"
    fn main() -> int {
        match 8 {
            8 => 50,
            _ => 33,
        }
    }"#;

    assert_eq!(llvm_exec!(src), 50);

    let src = r#"
    fn main() -> int {
        match 8 {
            30 => 50,
            _ => 34,
        }
    }"#;

    assert_eq!(llvm_exec!(src), 34);
}
