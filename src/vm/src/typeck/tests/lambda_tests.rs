use super::*;
use itertools::Itertools;

#[test]
fn check_simple_lambda_no_capture() {
    let tir = typeck!("fn () => 5;");
    let lines = lines!(&tir);
    assert_eq!(lines[0], "(λ() 5:number):fn()->number;");
}

#[test]
fn check_simple_lambda_with_parameter_no_capture() {
    let tir = typeck!("fn (x) => 5 + x;");
    let lines = lines!(&tir);
    assert_eq!(lines[0], "(λ($2:number) (+ 5:number $2:number):number):fn(number)->number;");
}

#[test]
fn check_fn_call() {
    let tir = typeck!("let f = fn(x) => x; f(3);");
    let lines = lines!(&tir);
    assert_eq!(
        lines[0],
        "let $1:fn(number)->number = (λ($4:number) $4:number):fn(number)->number;"
    );
    assert_eq!(lines[1], "($1:fn(number)->number 3:number):number;");
}

#[test]
fn check_lamda_with_capture() {
    let tir = typeck!("let x = 55; fn(y) => x + y;");
    let lines = lines!(&tir);
    assert_eq!(lines[0], "let $1:number = 55:number;");
    assert_eq!(lines[1], "(λ($6:number) (+ $1:number $6:number):number):fn(number)->number;");
}

#[test]
fn check_higher_order_lamda() {
    let tir = typeck!("let f = fn(x) => false; let g = fn(p) => p(3); g(f);");
    let lines = lines!(&tir);
    // note `false` is represented as `0`
    assert_eq!(lines[0], "let $1:fn(number)->bool = (λ($4:number) 0:bool):fn(number)->bool;");
    assert_eq!(
        lines[1],
        "let $9:fn(fn(number)->bool)->bool = (λ($12:fn(number)->bool) ($12:fn(number)->bool 3:number):bool):fn(fn(number)->bool)->bool;"
    );
    assert_eq!(lines[2], "($9:fn(fn(number)->bool)->bool $1:fn(number)->bool):bool;");
}
