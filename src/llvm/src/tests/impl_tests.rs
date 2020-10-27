use super::*;

#[test]
fn associated_fn() {
    let src = r#"
    struct S;

    impl S {
        fn five() -> int { 5 }
    }

    fn main() -> int {
        S::five()
    }
    "#;

    assert_eq!(llvm_exec!(src), 5);
}
