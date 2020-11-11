use super::*;

#[test]
fn occurs_check() {
    // f : ?0
    // ?0 = () -> ?0
    // ?0 = () -> () -> ?0
    expect_type_error_expr!("fn f() { f }; 5")
}

#[test]
fn check_lambda_wrong_arity_0() {
    expect_type_error_expr!("(fn () => 5)(3); 5");
}

#[test]
fn check_lambda_wrong_arity_1() {
    expect_type_error_expr!("(fn (x) => x)(3, 5); 5");
}

#[test]
fn check_call_non_function() {
    expect_type_error_expr!("5(5); 5");
}

#[test]
fn check_recursive_named_closure() {
    let _tir = typeck_expr!("fn f() { 1 + f() }; 5");
}

#[test]
fn check_simple_lambda_no_capture() {
    let tir = typeck_expr!("fn () => 5; 5");
    let lines = lines!(&tir);
    assert_eq!(lines[0], "(λ() 5:int):fn()->int;");
}

#[test]
fn check_simple_lambda_with_parameter_no_capture() {
    let tir = typeck_expr!("fn (x) => 5 + x; 5");
    let lines = lines!(&tir);
    assert_eq!(lines[0], "(λ(x:int) (+ 5:int x:int):int):fn(int)->int;");
}

#[test]
fn check_fn_call() {
    let tir = typeck_expr!("let f = fn(x) => x; f(3)");
    let lines = lines!(&tir);
    assert_eq!(lines[0], "let f:fn(int)->int = (λ(x:int) x:int):fn(int)->int;");
    assert_eq!(lines[1], "(f:fn(int)->int 3:int):int");
}

#[test]
fn check_lambda_with_capture() {
    let tir = typeck_expr!("let num = 55; fn(y) => num + y; num");
    let lines = lines!(&tir);
    assert_eq!(lines[0], "let num:int = 55:int;");
    assert_eq!(lines[1], "(λ(y:int) (+ num:int y:int):int):fn(int)->int;");
}

#[test]
fn check_higher_order_lambda() {
    let tir = typeck_expr!("let f = fn(x) => false; let g = fn(p) => p(3); g(f); 5");
    let lines = lines!(&tir);
    // note `false` is represented as `0`
    assert_eq!(lines[0], "let f:fn(int)->bool = (λ(x:int) false:bool):fn(int)->bool;");
    assert_eq!(
        lines[1],
        "let g:fn(fn(int)->bool)->bool = (λ(p:fn(int)->bool) (p:fn(int)->bool 3:int):bool):fn(fn(int)->bool)->bool;"
    );
    assert_eq!(lines[2], "(g:fn(fn(int)->bool)->bool f:fn(int)->bool):bool;");
}
