mod generics;
mod impls;
mod pattern;

/// just runs the compiler up to and including the ir lowering stage which includes resolution
macro resolve($src:expr) {{
    let driver = lc_driver::Driver::from_src($src);
    driver.gen_ir().unwrap();
}}

macro expect_resolution_error($src:expr) {{
    let driver = lc_driver::Driver::from_src($src);
    let _ = driver.gen_ir();
    // unwrapping ir is not sufficient as we continue with typechecking even if there are some
    // ir will only return an error if parsing fails
    // errors during resolution and lowering
    if !driver.has_errors() {
        panic!("expected error in resolution/lowering")
    }
}}

#[test]
fn resolve_redeclaration() {
    resolve!("fn main() -> int { let x = 5; let x = x; x }");
}

#[test]
fn self_in_free_function() {
    expect_resolution_error!("fn f(self) {}");
}
