use super::expect_error;

#[test]
fn check_assignment_to_immutable_var() {
    expect_error!("fn main() -> int { let x = 5; x = 7 }");
}
