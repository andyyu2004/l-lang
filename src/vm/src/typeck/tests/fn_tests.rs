use super::*;

#[test]
fn main_function_return_type_annotations() {
    let tir = typeck!("fn main() -> int { 5 }");
}
