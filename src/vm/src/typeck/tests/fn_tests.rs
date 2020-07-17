use super::*;

#[test]
fn main_function_return_type_annotations() {
    let tir = typeck_prog!("fn main() -> i32 { 5 }");
}
