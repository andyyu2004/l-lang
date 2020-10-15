use super::*;

#[test]
fn test_simple_mono() {
    let src = r#"
    fn id<T>(x: T) -> T { x }
    fn fst<T, U>(t: T, u: U) -> T { t }
    fn main() -> int { id(fst(5, false)) }"#;

    assert_eq!(llvm_exec!(src), 5);
}

#[test]
fn test_simple_mono2() {
    let src = r#"
    fn fst<T, U>(t: T, u: U) -> T { snd(u, t) }
    fn snd<T, U>(t: T, u: U) -> U { u }

    fn main() -> int {
        fst(5, true);
        fst(false, 9);
        fst(4, 9)
    }"#;

    assert_eq!(llvm_exec!(src), 4);
}

#[test]
fn test_mono_different_number_of_type_parameters() {
    let src = r#"
    fn fst<T, U>(t: T, u: U) -> T { snd(u, t) }
    fn snd<T, U>(t: T, u: U) -> U { u }

    fn main() -> int {
        fst(5, true);
        fst(false, 9);
        fst(4, 9)
    }"#;

    assert_eq!(llvm_exec!(src), 4);
}
