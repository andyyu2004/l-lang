mod pattern;

use crate::Driver;

/// just runs the compiler up to and including the ir lowering stage which includes resolution
macro resolve($src:expr) {{
    let driver = Driver::new($src);
    driver.gen_ir().unwrap();
}}

macro expect_error($src:expr) {{
    let driver = Driver::new($src);
    driver.gen_ir().unwrap_err();
}}

#[test]
fn resolve_redeclaration() {
    let _res = resolve!("fn main() -> int { let x = 5; let x = x; x }");
}
