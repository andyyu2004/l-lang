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
            .emit_loadu(0)
            .emit_loadl(0)
            .emit_op(Op::isub)
            .emit_op(Op::iret)
            .build();

        let main = CodeBuilder::default()
            // instruct the vm to create a closure
            .emit_iconst(2)
            // first parameter is the constant table index of the function
            // the variable we wish to close over is `x` which has relative index 0 on the stack
            .emit_closure(0, vec![(true, 0)])
            .emit_loadl(1)
            .emit_iconst(5)
            .emit_invoke(1)
            .emit_op(Op::iret)
            .build();

        let exec =
            Executable::new(vec![Function::with_upvalc(inner, 1).into()], Function::new(main));
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
        let inner = CodeBuilder::default().emit_istoreu_const(0, -4).emit_op(Op::iret).build();

        let main = CodeBuilder::default()
            // instruct the vm to create a closure
            .emit_iconst(5)
            .emit_closure(0, vec![(true, 0)])
            .emit_loadl(1)
            .emit_invoke(0)
            .emit_op(Op::pop) // pop the return of the closure
            .emit_loadl(0)
            .emit_op(Op::iret)
            .build();

        let exec =
            Executable::new(vec![Function::with_upvalc(inner, 1).into()], Function::new(main));
        let mut vm = VM::with_default_gc(exec);
        let ret = vm.run()?;
        assert_eq!(ret, Val::Int(-4));

        Ok(())
    }

    /// fn main() -> i64 {
    ///     let x = -5;
    ///     let outer = fn() -> i64 => {
    ///         let inner = fn() -> i64 => x;
    ///         inner()
    ///     }
    ///     outer()
    /// }
    #[test]
    fn nested_closures() -> VMResult<()> {
        let main = CodeBuilder::default()
            .emit_iconst(-9)
            .emit_closure(0, vec![(true, 0)])
            .emit_loadl(1)
            .emit_invoke(0)
            .emit_op(Op::iret)
            .build();
        let outer = CodeBuilder::default()
            .emit_closure(1, vec![(false, 0)])
            .emit_loadl(0)
            .emit_invoke(0)
            .emit_op(Op::iret)
            .build();
        let inner = CodeBuilder::default().emit_loadu(0).emit_op(Op::iret).build();

        let exec = Executable::new(
            vec![Function::with_upvalc(outer, 1).into(), Function::with_upvalc(inner, 1).into()],
            Function::new(main),
        );

        let mut vm = VM::with_default_gc(exec);
        let ret = vm.run()?;
        assert_eq!(ret, Val::Int(-9));

        Ok(())
    }

    /// fn main() -> i64 {
    ///     let x = -20;
    ///     let outer = fn() -> fn() -> i64 => {
    ///         let inner = fn() -> i64 => x - 1;
    ///         inner
    ///     }
    ///     let f = outer();
    ///     f()
    /// }
    ///
    /// assert(main(), -21);
    ///
    #[test]
    fn return_closure() -> VMResult<()> {
        let main = CodeBuilder::default()
            .emit_iconst(-20)
            .emit_closure(0, vec![(true, 0)])
            .emit_loadl(1)
            .emit_invoke(0) // this will leave `inner` on the stack
            .emit_invoke(0)
            .emit_op(Op::ret)
            .build();
        let outer = CodeBuilder::default()
            .emit_closure(1, vec![(false, 0)])
            .emit_loadl(0)
            .emit_op(Op::rret)
            .build();
        let inner = CodeBuilder::default()
            .emit_loadu(0)
            .emit_iconst(1)
            .emit_op(Op::isub)
            .emit_op(Op::iret)
            .build();

        let exec = Executable::new(
            vec![Function::with_upvalc(outer, 1).into(), Function::with_upvalc(inner, 1).into()],
            Function::new(main),
        );

        let mut vm = VM::with_default_gc(exec);
        let ret = vm.run()?;
        assert_eq!(ret, Val::Int(-21));

        Ok(())
    }

    /// fn main() -> i64 {
    ///     let outer = fn() -> fn() -> i64 => {
    ///         let x = -20;
    ///         let inner = fn() -> i64 => x - 1;
    ///         inner
    ///     }
    ///     let f = outer();
    ///     f()
    /// }
    ///
    /// assert(main(), -21);
    ///
    #[test]
    fn closed_upvalues() -> VMResult<()> {
        let main = CodeBuilder::default()
            .emit_ldc(0)
            .emit_loadl(0)
            .emit_invoke(0)
            // emit a few constants to overwrite where the closed upvalue is pointing to
            .emit_iconst(-99)
            .emit_iconst(-99)
            .emit_iconst(-99)
            .emit_loadl(1)
            .emit_invoke(0)
            .emit_op(Op::ret)
            .build();
        let outer = CodeBuilder::default()
            .emit_iconst(-20)
            .emit_closure(1, vec![(true, 0)])
            .emit_loadl(1)
            .emit_close_upvalue(0)
            .emit_op(Op::rret)
            .build();
        let inner = CodeBuilder::default()
            .emit_loadu(0)
            .emit_iconst(1)
            .emit_op(Op::isub)
            .emit_op(Op::iret)
            .build();

        let exec = Executable::new(
            vec![Function::new(outer).into(), Function::with_upvalc(inner, 1).into()],
            Function::new(main),
        );

        let mut vm = VM::with_default_gc(exec);
        let ret = vm.run()?;
        assert_eq!(ret, Val::Int(-21));

        Ok(())
    }

    /// tests that sibling closures capture the same variable and can see mutations
    /// i.e. inner should see the mutation performed by inner_mut
    /// fn main() -> i64 {
    ///     let outer = fn() -> fn() -> i64 => {
    ///         let x = -20;
    ///         let inner = fn() -> i64 => x;
    ///         let inner_mut = fn() -> i64 => x = -50;
    ///         inner_mut();
    ///         inner
    ///     }
    ///     outer()()
    /// }
    ///
    /// assert(main(), -21);
    ///
    #[test]
    fn sibling_upvalues() -> VMResult<()> {
        let main = CodeBuilder::default()
            .emit_ldc(0)
            .emit_loadl(0)
            .emit_invoke(0)
            .emit_invoke(0)
            .emit_op(Op::iret)
            .build();
        let outer = CodeBuilder::default()
            .emit_iconst(-20)
            .emit_closure(1, vec![(true, 0)])
            .emit_closure(2, vec![(true, 0)])
            .emit_loadl(2)
            .emit_invoke(0)
            .emit_op(Op::pop)
            .emit_loadl(1)
            .emit_op(Op::rret)
            .build();
        let inner = CodeBuilder::default().emit_loadu(0).emit_op(Op::iret).build();
        let inner_mut = CodeBuilder::default().emit_istoreu_const(0, -50).emit_op(Op::iret).build();

        let exec = Executable::new(
            vec![
                Function::new(outer).into(),
                Function::with_upvalc(inner, 1).into(),
                Function::with_upvalc(inner_mut, 1).into(),
            ],
            Function::new(main),
        );

        let mut vm = VM::with_default_gc(exec);
        let ret = vm.run()?;
        assert_eq!(ret, Val::Int(-50));

        Ok(())
    }

    /// fn main() -> i64 {
    ///     let outer = fn() -> fn() -> i64 => {
    ///         var x = 0;
    ///         let inner = fn() -> i64 => x += 1;
    ///         inner
    ///     }
    ///     let f = outer();
    ///     f();
    ///     f();
    ///     f()
    /// }
    ///
    /// assert(main(), 2);
    #[test]
    fn complex_closures() -> VMResult<()> {
        let main = CodeBuilder::default()
            .emit_ldc(0)
            .emit_loadl(0)
            .emit_invoke(0)
            .emit_loadl(1)
            .emit_invoke(0)
            .emit_op(Op::pop)
            .emit_loadl(1)
            .emit_invoke(0)
            .emit_op(Op::pop)
            .emit_loadl(1)
            .emit_invoke(0)
            .emit_op(Op::iret)
            .build();
        let outer = CodeBuilder::default()
            .emit_iconst(0)
            .emit_closure(1, vec![(true, 0)])
            .emit_loadl(1)
            .emit_close_upvalue(0)
            .emit_op(Op::rret)
            .build();

        let inner = CodeBuilder::default()
            .emit_loadu(0)
            .emit_iconst(1)
            .emit_op(Op::iadd)
            .emit_storeu(0)
            .emit_op(Op::iret)
            .build();

        let exec = Executable::new(
            vec![Function::new(outer).into(), Function::with_upvalc(inner, 1).into()],
            Function::new(main),
        );

        let mut vm = VM::with_default_gc(exec);
        let ret = vm.run()?;
        assert_eq!(ret, Val::Int(3));

        Ok(())
    }
}
