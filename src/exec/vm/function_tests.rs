#[cfg(test)]
mod test {
    use crate::error::VMResult;
    use crate::{compiler::Executable, exec::*};

    /// fn f(x: i64, y: i64) -> i64 {
    ///     x - y
    /// }
    ///
    /// fn main() -> i64 {
    ///     let a = 2;
    ///     let b = 3;
    ///     f(a, b)
    /// }
    ///
    /// assert_eq!(main(), -1);
    ///
    #[test]
    fn simple_function_call() -> VMResult<()> {
        let f = CodeBuilder::default()
            .emit_iloadl(0)
            .emit_iloadl(1)
            .emit_op(Op::isub)
            .emit_op(Op::ret)
            .build();
        let main = CodeBuilder::default()
            .emit_ldc(0)
            .emit_iconst(2)
            .emit_iconst(3)
            .emit_invoke(2)
            .emit_op(Op::ret)
            .build();

        let exec = Executable::new(vec![Function::new(f).into()], Function::new(main));
        let mut vm = VM::with_default_gc(exec);
        let ret = vm.run()?;
        assert_eq!(ret, Val::Int(-1));

        Ok(())
    }
    ///
    /// fn f(x: i64) -> i64 { x - 1 }
    ///
    /// fn main() -> i64 {
    ///     let a = 2;
    ///     f(a);
    ///     f(a);
    ///     f(a)
    /// }
    ///
    /// assert_eq!(main(), -1);
    ///
    #[test]
    fn multiple_flat_function_calls() -> VMResult<()> {
        let f = CodeBuilder::default()
            .emit_iloadl(0)
            .emit_iconst(1)
            .emit_op(Op::isub)
            .emit_op(Op::ret)
            .build();
        let main = CodeBuilder::default()
            .emit_iconst(2)
            .emit_ldc(0)
            .emit_iloadl(0)
            .emit_invoke(1)
            .emit_op(Op::pop)
            .emit_ldc(0)
            .emit_iloadl(0)
            .emit_invoke(1)
            .emit_op(Op::pop)
            .emit_ldc(0)
            .emit_iloadl(0)
            .emit_invoke(1)
            .emit_op(Op::ret)
            .build();

        let exec = Executable::new(vec![Function::new(f).into()], Function::new(main));
        let mut vm = VM::with_default_gc(exec);
        let ret = vm.run()?;
        assert_eq!(ret, Val::Int(1));

        Ok(())
    }
}
