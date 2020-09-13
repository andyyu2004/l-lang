use super::*;

#[test]
fn test_simple_deref() {
    typeck_expr!("*(box 5)");
}
