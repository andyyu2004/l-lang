//! generics and parametric polymorphism

use super::*;

#[test]
fn simple_generic_struct() {
    let src = r#"
    struct G<T> {
        t: T
    }

    fn main() -> int {
        let g = G { t: 5 };
        g.t
    }"#;

    typeck!(src);
}

#[test]
fn check_generic_parameters_are_the_same() {
    let src = r#"
    struct G<T> {
        x: T,
        y: T,
    }

    fn main() -> int {
        let g = G { x: 5, y: false };
        g.x
    }"#;

    expect_error!(src);
}

#[test]
fn generic_functions_and_structs() {
    let src = r#"
    struct G<T> {
        t: T
    }

    fn id<T>(x: T) -> T {
        x
    }

    fn main() -> int {
        let g = G { t: id(6) };
        g.t
    }"#;

    typeck!(src);
}

#[test]
fn multiple_generic_parameters() {
    let src = r#"
    struct G<T, U> {
        t: T,
        u: U,
    }

    fn id<T>(x: T) -> T {
        x
    }

    fn main() -> int {
        let g = G { t: id(6), u: id(false) };
        g.t
    }"#;

    typeck!(src);
}

#[test]
fn nested_generic_structs() {
    let src = r#"
    struct G<T, U> {
        s: S<U>,
        u: U,
    }

    struct S<T> {
        t: T
    }

    struct R<K> {
        k: K,
    }


    fn main() -> int {
        let s = S { t: 5 };
        let g = G {
            s,
            u: R { k: bool }
        };
        g.s.t
    }"#;

    typeck!(src);
}
