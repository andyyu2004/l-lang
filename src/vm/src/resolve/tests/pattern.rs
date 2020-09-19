use super::*;

// #[test]
fn identifier_bound_more_than_once_in_pattern() {
    let src = "fn main() -> int {
        let (x, x) = (1,2); 5
    }";
    expect_error!(src);
}
