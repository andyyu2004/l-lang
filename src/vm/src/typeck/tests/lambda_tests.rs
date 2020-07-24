use super::*;
use itertools::Itertools;

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
    // note `false` is represented as `0`
    assert_eq!(lines[0], "let $1:(number)->bool = (λ($4:number) 0:bool):(number)->bool;");
    assert_eq!(
        lines[1],
        "let $9:((number)->bool)->bool = (λ($12:(number)->bool) ($12:(number)->bool 3:number):bool):((number)->bool)->bool;"
    );
    assert_eq!(lines[2], "($9:((number)->bool)->bool $1:(number)->bool):bool;");
}
