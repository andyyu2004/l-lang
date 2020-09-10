use super::{expect_error, typeck};

// tuple tests are also here as they are similar to structs in many aspects
#[test]
fn check_tuple_out_of_bounds() {
    expect_error!("fn main() -> int { (1,2).2 }")
}

#[test]
fn check_struct_badly_typed() {
    expect_error!("struct S { x: int } fn main() { S { x: false }; 5 }");
}

#[test]
fn check_struct_missing_fields() {
    expect_error!("struct S { x: int, y: bool } fn main() { S { x: 3 }; 5 }");
}

#[test]
fn check_struct_set_field_multiple() {
    simple_logging::log_to_file("log.txt", log::LevelFilter::Info).unwrap();
    expect_error!("struct S { x: int } fn main() { S { x: 4, x: 5 }; 5 }");
}

#[test]
fn check_struct_differing_orders() {
    let src = r#"
    struct S { x: int, y: bool }
    fn main() {
        S {
            y: false,
            x: 4,
        };
    }
    "#;
    typeck!(src);
}
