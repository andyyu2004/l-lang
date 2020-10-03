mod pattern;

use crate::Driver;

/// just runs the compiler up to and including the ir lowering stage which includes resolution
macro resolve($src:expr) {{
    let driver = Driver::new($src);
    driver.gen_ir().unwrap();
}}

macro expect_error($src:expr) {{
    let driver = Driver::new($src);
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
    let _res = resolve!("fn main() -> int { let x = 5; let x = x; x }");
}
