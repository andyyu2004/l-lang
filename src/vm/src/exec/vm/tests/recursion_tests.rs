use crate::error::VMResult;
use crate::{compiler::Executable, exec::*};

#[test]
fn run_fib() {
    let src = r#"
    fn main() -> number { fib(10) }

    fn fib(n: number) -> number {
        if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
    }
    "#;
    let val = crate::exec(src).unwrap();
    assert_eq!(val, Val::Double(55.0));
}
