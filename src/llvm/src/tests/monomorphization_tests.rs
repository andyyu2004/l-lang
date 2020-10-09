use super::*;

#[test]
fn test_simple_mono() {
    let src = r#"
    fn id<T>(x: T) -> T { x }
    fn fst<T, U>(t: T, u: U) -> T { t }
    fn main() -> int { id(fst(5, false)) }"#;

    assert_eq!(llvm_exec!(src), 5);
}

// #[test]
fn test_simple_mono2() {
    let src = r#"
    fn fst<T, U>(t: T, u: U) -> T { snd(u, t) }
    fn snd<T, U>(t: T, u: U) -> U { u }

    fn main() -> int {
        fst(4, false)
    }"#;

    assert_eq!(llvm_exec!(src), 4);
}
