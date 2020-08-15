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
fn llvm_side_effects() {
    let src = r#"
    fn main() -> number {
        let mut x = 0;
        x = x + 1;
        x
    }"#;
    assert_eq!(llvm_exec!(src), 1.0);
}

#[test]
fn llvm_multiple_returns() {
    let src = r#"
    fn main() -> number {
        return 5;
        return 6;
        return 7;
    }"#;
    assert_eq!(llvm_exec!(src), 5.0);
}

#[test]
fn llvm_fib_all_explicit_returns() {
    let src = r#"
    fn main() -> number { return fib(10); }

    fn fib(n: number) -> number {
        return if n < 2 { return n; } else { return fib(n - 1) + fib(n - 2); };
    }
    "#;
    assert_eq!(llvm_exec!(src), 55.0)
}

#[test]
fn llvm_fib_mixed_returns() {
    let src = r#"
    fn main() -> number { return fib(10); }

    fn fib(n: number) -> number {
        // note one branch is explicit return and one is not
        return if n < 2 { n } else { return fib(n - 1) + fib(n - 2); };
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
