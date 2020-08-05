//! simply tests for the expected output
use super::llvm_exec;

#[test]
fn llvm_fib() {
    let src = r#"
    fn main() -> number { fib(10) }

    fn fib(n: number) -> number {
        if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
    }
    "#;
    assert_eq!(llvm_exec!(src), 55.0)
}

#[test]
fn llvm_vars() {
    let src = r#"
    fn main() -> number {
        let x = 2;
        let y = 4;
        x + y
    }
    "#;
    assert_eq!(llvm_exec!(src), 6.0)
}

#[test]
fn llvm_lambda_no_capture() {
    let src = r#"
    fn main() -> number {
        let f = fn() => 5;
        2 + f()
    }
    "#;
    assert_eq!(llvm_exec!(src), 7.0)
}
