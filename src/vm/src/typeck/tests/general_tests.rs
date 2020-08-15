use super::typeck;

#[test]
#[should_panic]
fn check_assignment_to_immutable_var() {
    typeck!("fn main() -> number { let x = 5; x = 7 }");
}
