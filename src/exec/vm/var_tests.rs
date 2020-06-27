#[cfg(test)]
mod test {
    use crate::error::VMResult;
    use crate::{compiler::Executable, exec::*};

    /// fn main() -> i64 {
    ///     var x = 2;
    ///     x = -5;
    ///     x
    /// }
    ///
    /// assert_eq!(main(), -5);
    ///
    #[test]
    fn test_access_set_local() -> VMResult<()> {
        let main = CodeBuilder::default()
            .emit_iconst(2)
            .emit_istorel(0, -5)
            .emit_op(Op::iret)
            .build();

        let exec = Executable::from(Function::new(main));
        let mut vm = VM::with_default_gc(exec);
        let ret = vm.run()?;
        assert_eq!(ret, Val::Int(-5));

        Ok(())
    }
}
