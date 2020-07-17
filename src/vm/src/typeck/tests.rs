use crate::driver::Driver;
use crate::wrap_in_main;

macro typeck($src:expr) {{
    let driver = Driver::new(&wrap_in_main($src));
    let tir = driver.gen_tir().unwrap();
    tir.to_string()
}}

fn remove_surrounding_block(s: &str) -> &str {
    s.lines().nth(1).unwrap().trim()
}

#[test]
fn check_simple_lambda_no_capture() {
    let tir = typeck!("fn () => 5;");
    let output = remove_surrounding_block(&tir);
    assert_eq!(output, "(λ() 5:number):() -> number;");
}

#[test]
fn check_simple_lambda_with_parameter_no_capture() {
    let tir = typeck!("fn (x) => 5 + x;");
    let output = remove_surrounding_block(&tir);
    assert_eq!(output, "(λ($2:number) (+ 5:number $2:number):number):(number) -> number;");
}

fn with_capture() {
    // let x = 55;
    // fn(y) => x + y + z;
}
