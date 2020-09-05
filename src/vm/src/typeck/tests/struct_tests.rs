use super::typeck;

#[should_panic]
#[test]
fn check_struct_badly_typed() {
    typeck!("struct S { x: int } fn main() { S { x: false }; 5 }");
}

#[should_panic]
#[test]
fn check_struct_missing_fields() {
    typeck!("struct S { x: int, y: bool } fn main() { S { x: 3 }; 5 }");
}

#[should_panic]
#[test]
fn check_struct_set_field_multiple() {
    typeck!("struct S { x: int } fn main() { S { x: 4, x: 5 }; 5 }");
}
