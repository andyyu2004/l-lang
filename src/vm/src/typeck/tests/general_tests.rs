use super::typeck;

#[test]
#[should_panic]
fn check_assignment_to_immutable_var() {
    typeck!("fn main() -> int { let x = 5; x = 7 }");
}
