use super::*;

#[test]
fn detect_use_of_uninit_simple() {
    let src = r#"
    fn main() -> int {
        let x;
        x
    }"#;

    expect_analysis_error!(src);
}
