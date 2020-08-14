use super::typeck_prog;

#[should_panic]
#[test]
fn check_struct_badly_typed() {
    typeck_prog!("struct S { x: number } fn main() { S { x: false }; 5 }");
}

#[should_panic]
#[test]
fn check_struct_missing_fields() {
    typeck_prog!("struct S { x: number, y: bool } fn main() { S { x: 3 }; 5 }");
}

#[should_panic]
#[test]
fn check_struct_set_field_multiple() {
    typeck_prog!("struct S { x: number } fn main() { S { x: 4, x: 5 }; 5 }");
}
