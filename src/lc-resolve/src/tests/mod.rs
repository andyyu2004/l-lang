mod generics;
mod impls;
mod macros;
mod pattern;

use lc_util::stringify_tt;

/// just runs the compiler up to and including the ir lowering stage which includes resolution
#[macro_export]
macro_rules! resolve {
    ($src:tt) => {{
        let src = lc_util::stringify_tt!($src);
        let driver = lc_driver::Driver::from_src(src);
        driver.gen_ir().unwrap();
    }};
}

#[macro_export]
macro_rules! expect_resolution_error {
    ($src:tt) => {{
        let src = stringify_tt!($src);
        let driver = lc_driver::Driver::from_src(src);
        let _ = driver.gen_ir();
        // unwrapping ir is not sufficient as we continue with typechecking even if there are some
        // ir will only return an error if parsing fails
        // errors during resolution and lowering
        if !driver.has_errors() {
            panic!("expected error in resolution/lowering")
        }
    }};
}

#[test]
fn resolve_redeclaration() {
    resolve!({
        fn main() -> int {
            let x = 5;
            let x = x;
            x
        }
    });
}

#[test]
fn self_in_free_function() {
    expect_resolution_error!({
        fn f(self) {
        }
    });
}
