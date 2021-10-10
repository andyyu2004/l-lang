use super::*;

// check collection works when items are declared out of order
#[test]
fn test_out_of_order_decl() {
    let src = r#"
    fn f(s: S) {}
    struct S { }
    fn main() -> int { 5 }
    "#;

    typeck!(src);
}
