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
fn incorrect_number_of_generic_args() {
    let src = r#"
    struct S<T> { t: T }
    fn main() -> int {
        let s: S<_, _>;
        5
    }"#;
    expect_error!(src);
}

#[test]
fn conflicting_generic_args_in_path() {
    let src = r#"
    struct S<T> {
        t: T
    }

    fn main() -> int {
        let s: S<int> = S { t: false };
        s.t
    }"#;

    expect_error!(src);
}

#[test]
fn infer_generic_args_in_path() {
    let src = r#"
    struct S<T> {
        t: T
    }

    fn main() -> int {
        let s: S<_> = S { t: 5 };
        s.t
    }"#;

    typeck!(src);
}

#[test]
fn incorrect_number_of_generic_args_in_struct_decl() {
    // TODO need to fix
    let src = r#"
    struct S<T> {
        t: T
    }

    struct K<T, U> {
        s: S<T, U>
    }
    "#;

    expect_error!(src);
}

#[test]
fn more_complex_parametrix_inference() {
    let src = r#"
    enum Either<L, R> {
        Left(L),
        Right(R),
    }

    fn main() -> int {
        let either: Either<int, _> = Either::Right(false);
        5
    }"#;

    typeck!(src);

    // check that the `U` parameter of Complex<T, U> correctly
    // constraints the type of `L`
    let src = r#"
    enum Option<T> {
        Some(T),
        None,
    }

    enum Either<L, R> {
        Left(L),
        Right(R),
    }

    struct Complex<T, U> {
        either: Either<U, T>,
        option: Option<U>,
    }

    fn main() -> int {
        let option: Option<_> = Option::Some(false);
        let either: Either<_, _> = Either::Right(9);

        Complex { either, option };
        5
    }"#;

    typeck!(src);
}

#[test]
fn nested_generic_structs() {
    let src = r#"
    struct G<T, U> {
        s: S<U>,
        u: T,
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
            u: R { k: false }
        };
        g.s.t
    }"#;

    let prog = typeck!(src);
    eprintln!("{}", prog);
}
