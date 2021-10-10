use super::*;

#[test]
fn check_boolean_match_exhaustive() {
    let src = r#"
    fn f(b: bool) -> int {
        match b {
            false => 0,
            true  => 1,
        }
    }
    "#;

    analyze!(src);
}

#[test]
fn check_boolean_match_nonexhaustive() {
    let src = r#"
    fn f(b: bool) -> int {
        match b {
            true  => 1,
        }
    }
    "#;

    expect_analysis_error!(src);
}

#[test]
fn check_adt_match_nonexhaustive() {
    let src = r#"
    enum E<T, U> {
        A,
        B(T),
        C(U),
    }

    fn f<T>(b: E<T, int>) -> int {
        match b {
            E::A => 5,
            E::C(u) => u,
        }
    }
    "#;

    expect_analysis_error!(src);
}

#[test]
fn check_nonexhaustive_box_pattern() {
    let src = r#"
    enum Option<T> {
        Some(T),
        None,
    }

    fn f() -> int {
        let opt = box Option::None;
        match opt {
            &Option::Some(x) => x,
        }
    }
    "#;

    expect_analysis_error!(src);
}
