use super::*;
use crate::expect_resolution_error;

#[test]
fn resolve_undeclared_generic_parameter() {
    expect_resolution_error!({
        fn f(t: T) -> T {
            t
        }
    });
}

#[test]
fn resolve_undeclared_generic_parameter_in_extern() {
    expect_resolution_error!({
        extern "C" {
            fn f<T>(t: T) -> T;
            fn g(t: &T) -> T;
        }
    });
}
