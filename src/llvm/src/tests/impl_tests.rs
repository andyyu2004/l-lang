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

#[test]
fn e() {
    let src = r#"
    fn main() -> int {
        let s = S::new(5, false);
        s.u
    }

    struct S<T, U> {
        t: T,
        u: U,
    }

    impl<T, U> S<T, U> {
        fn new(u: U, t: T) -> Self {
            Self { t, u }
        }
    }
    "#;

    assert_eq!(llvm_exec!(src), 5);
}
