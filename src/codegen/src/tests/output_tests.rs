//! simply tests for the expected output

use super::*;

#[test]
fn llvm_invalid_main_type() {
    let src = "fn main() {}";
    llvm_jit_expect_error!(src);
}

#[test]
fn llvm_construct_empty_struct() {
    let src = "struct S; fn main() -> int { S; 0 }";
    llvm_jit!(src);
}

#[test]
fn llvm_fib() {
    let src = r#"
    fn main() -> int { fib(10) }

    fn fib(n: int) -> int {
        if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
    }
    "#;
    assert_eq!(llvm_jit!(src), 55)
}

#[test]
fn llvm_tuple() {
    let src = r#"
    fn main() -> int {
        mktuple();
        0
    }

    fn mktuple() -> (int, bool) {
        (30, true)
    }
    "#;

    llvm_jit!(src);
}

#[test]
fn llvm_assignment_value() {
    let src = r#"
    fn main() -> int {
        let mut x = 0;
        x = 6
    }
    "#;
    assert_eq!(llvm_jit!(src), 6);
}

#[test]
fn llvm_chained_assignment() {
    // check the value of the assignment,
    // as well as the fact that both `x` and `y` get assigned to

    let src = r#"
    fn main() -> int {
        let mut x = 0;
        let mut y = 0;
        x = y = 6
    }
    "#;
    assert_eq!(llvm_jit!(src), 6);

    let src = r#"
    fn main() -> int {
        let mut x = 0;
        let mut y = 0;
        x = y = 6;
        y
    }
    "#;
    assert_eq!(llvm_jit!(src), 6);

    let src = r#"
    fn main() -> int {
        let mut x = 0;
        let mut y = 0;
        x = y = 6;
        x
    }
    "#;
    assert_eq!(llvm_jit!(src), 6);
}

#[test]
fn llvm_missing_main() {
    llvm_jit_expect_error!("");
}

/// checks that the expression statements are actually being performed, even though their results
/// (`BasicValueEnum`) are ignored during compilation
#[test]
fn llvm_side_effects() {
    let src = r#"
    fn main() -> int {
        let mut x = 0;
        x = x + 1;
        x
    }"#;
    assert_eq!(llvm_jit!(src), 1);
}

#[test]
fn llvm_multiple_returns() {
    let src = r#"
    fn main() -> int {
        return 5;
        return 6;
        return 7;
    }"#;
    assert_eq!(llvm_jit!(src), 5);
}

// #[test]
// fn llvm_non_escaping_closure() {
//     let src = r#"
//     fn main() -> int {
//         let x = 5;
//         (fn () => x + 4)()
//     }
//     "#;
//     assert_eq!(llvm_jit!(src), 9);
// }

// #[test]
// fn llvm_lambda_no_capture() {
//     let src = r#"
//     fn main() -> int {
//         let f = fn() => 5;
//         2 + f()
//     }
//     "#;
//     assert_eq!(llvm_jit!(src), 7)
// }

#[test]
fn llvm_fib_all_explicit_returns() {
    let src = r#"
    fn main() -> int { return fib(10) }

    fn fib(n: int) -> int {
        return if n < 2 { return n } else { return fib(n - 1) + fib(n - 2) }
    }
    "#;
    assert_eq!(llvm_jit!(src), 55)
}

#[test]
fn llvm_fib_mixed_returns() {
    let src = r#"
    fn main() -> int { return fib(10) }

    fn fib(n: int) -> int {
        // note one branch is explicit return and one is not
        return if n < 2 { n } else { return fib(n - 1) + fib(n - 2) }
    }
    "#;
    assert_eq!(llvm_jit!(src), 55)
}

#[test]
fn llvm_vars() {
    let src = r#"
    fn main() -> int {
        let x = 2;
        let y = 4;
        x + y
    }
    "#;
    assert_eq!(llvm_jit!(src), 6)
}
