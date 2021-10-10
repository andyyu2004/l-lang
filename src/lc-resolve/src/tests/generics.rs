use super::*;

#[test]
fn resolve_undeclared_generic_parameter() {
    let src = "fn f(t: T) -> T { t }";
    expect_resolution_error!(src);
}

#[test]
fn resolve_undeclared_generic_parameter_in_extern() {
    let src = r#"
    extern {
        fn f<T>(t: T) -> T;
        fn g(t: &T) -> T;
    }
    "#;
    expect_resolution_error!(src);
}
