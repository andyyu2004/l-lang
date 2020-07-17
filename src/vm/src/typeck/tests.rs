use crate::driver::Driver;
use crate::wrap_in_main;
use itertools::Itertools;

macro typeck($src:expr) {{
    let driver = Driver::new(&wrap_in_main($src));
    let tir = driver.gen_tir().unwrap();
    tir.to_string()
}}

macro lines($s:expr) {{
    let mut lines = $s.lines();
    lines.next().unwrap();
    lines.next_back().unwrap();
    lines.map(|line| line.trim()).collect_vec()
}}

// only works for single line expressions
fn remove_surrounding_block(s: &str) -> &str {
    s.lines().nth(1).unwrap().trim()
}

#[test]
fn check_simple_lambda_no_capture() {
    let tir = typeck!("fn () => 5;");
    let output = remove_surrounding_block(&tir);
    assert_eq!(output, "(λ() 5:number):()->number;");
}

#[test]
fn check_simple_lambda_with_parameter_no_capture() {
    let tir = typeck!("fn (x) => 5 + x;");
    let output = remove_surrounding_block(&tir);
    assert_eq!(output, "(λ($2:number) (+ 5:number $2:number):number):(number)->number;");
}

#[test]
fn check_fn_call() {
    let tir = typeck!("let f = fn(x) => x; f(3);");
    let lines = lines!(&tir);
    assert_eq!(lines[0], "let $1:(number)->number = (λ($4:number) $4:number):(number)->number;");
    assert_eq!(lines[1], "($1:(number)->number 3:number):number;");
}

#[test]
fn check_lamda_with_capture() {
    let tir = typeck!("let x = 55; fn(y) => x + y;");
    let lines = lines!(&tir);
    assert_eq!(lines[0], "let $1:number = 55:number;");
    assert_eq!(lines[1], "(λ($6:number) (+ $1:number $6:number):number):(number)->number;");
}

#[test]
fn check_higher_order_lamda() {
    let tir = typeck!("let f = fn(x) => false; let g = fn(p) => p(3); g(f);");
    let lines = lines!(&tir);
    assert_eq!(lines[0], "let $1:(number)->bool = (λ($4:number) false:bool):(number)->bool;");
    assert_eq!(
        lines[1],
        "let $9:((number)->bool)->bool = (λ($12:(number)->bool) ($12:(number)->bool 3:number):bool):((number)->bool)->bool;"
    );
    assert_eq!(lines[2], "($9:((number)->bool)->bool $1:(number)->bool):bool;");
}
