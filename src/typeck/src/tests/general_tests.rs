use super::*;

// #[test]
// fn check_assignment_to_immutable_var() {
//     expect_error!("fn main() -> int { let x = 5; x = 7 }");
// }

#[test]
fn check_box_expr() {
    let _tir = typeck!("fn main() -> int { let x = 5; let boxed = box x; 5 }");
    // dbg!(_tir);
}
