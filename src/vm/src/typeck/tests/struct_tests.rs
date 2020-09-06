use super::{expect_error, typeck};

// #[test]
// fn check_struct_badly_typed() {
//     expect_error!("struct S { x: int } fn main() { S { x: false }; 5 }");
// }

// #[test]
// fn check_struct_missing_fields() {
//     expect_error!("struct S { x: int, y: bool } fn main() { S { x: 3 }; 5 }");
// }

// #[test]
// fn check_struct_set_field_multiple() {
//     expect_error!("struct S { x: int } fn main() { S { x: 4, x: 5 }; 5 }");
// }
