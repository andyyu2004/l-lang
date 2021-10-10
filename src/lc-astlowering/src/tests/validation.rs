use super::*;

#[test]
fn function_without_body() {
    let src = "fn main();";
    expect_lowering_error!(src);
}
