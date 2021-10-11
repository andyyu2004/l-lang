use super::*;
use crate::expect_resolution_error;

#[test]
fn identifier_bound_more_than_once_in_pattern() {
    expect_resolution_error!({
        fn main() -> int {
            let (x, x) = (1, 2);
            5
        }
    });
}
