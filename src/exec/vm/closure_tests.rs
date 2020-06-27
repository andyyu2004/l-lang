#[cfg(test)]
mod test {
    use crate::error::VMResult;
    use crate::{compiler::Executable, exec::*};

    /// fn main() -> i64 {
    ///     let x = 2;
    ///     let inner = fn(y: i64) => x - y;
    ///     inner(5)
    /// }
    ///
    /// assert_eq!(main(), -3);
    ///
    #[test]
    fn load_nonlocal_var() -> VMResult<()> {
        let inner = CodeBuilder::default()
            .emit_iloadu(0)
            .emit_iloadl(0)
            .emit_op(Op::isub)
            .emit_op(Op::iret)
            .build();

        let main = CodeBuilder::default()
            // instruct the vm to create a closure
            .emit_iconst(2)
            // first parameter is the constant table index of the function
            // the variable we wish to close over is `x` which has relative index 0 on the stack
            .emit_closure(0, vec![(true, 0)])
            .emit_iconst(5)
            .emit_invoke(1)
            .emit_op(Op::iret)
            .build();

        let exec = Executable::new(
            vec![Function::with_upvalc(inner, 1).into()],
            Function::new(main),
        );
        let mut vm = VM::with_default_gc(exec);
        let ret = vm.run()?;
        assert_eq!(ret, Val::Int(-3));

        Ok(())
    }

    /// fn main() -> i64 {
    ///     var x = 5;
    ///     let inner = fn() -> i64 => x = -4;
    ///     inner();
    ///     x
    /// }
    ///
    /// assert(main(), -4);
    ///
    #[test]
    fn store_nonlocal_var() -> VMResult<()> {
        let inner = CodeBuilder::default()
            .emit_istoreu(0, -4)
            .emit_op(Op::iret)
            .build();

        let main = CodeBuilder::default()
            // instruct the vm to create a closure
            .emit_iconst(5)
            .emit_closure(0, vec![(true, 0)])
            .emit_invoke(0)
            .emit_op(Op::pop) // pop the return of the closure
            .emit_iloadl(0)
            .emit_op(Op::iret)
            .build();

        let exec = Executable::new(
            vec![Function::with_upvalc(inner, 1).into()],
            Function::new(main),
        );
        let mut vm = VM::with_default_gc(exec);
        let ret = vm.run()?;
        assert_eq!(ret, Val::Int(-4));

        Ok(())
    }
}
