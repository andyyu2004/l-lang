/// note the two first allocations are for the start function and the main function
#[cfg(test)]
mod test {
    use crate::compiler::Executable;
    use crate::error::VMResult;
    use crate::*;

    #[test]
    fn it_works() -> VMResult<()> {
        let main_code = CodeBuilder::default()
            .emit_iconst(5)
            .emit_op(Op::ret)
            .build();
        let executable = Executable::from(Function::new(main_code));
        let mut vm = VM::new(executable);
        let ret = vm.run()?;
        assert_eq!(ret, Val::Prm(5));
        Ok(())
    }

    /// tests the vm applies arithmetic left to right
    #[test]
    fn vm_arith_order() -> VMResult<()> {
        let code = CodeBuilder::default()
            .emit_op(Op::iconst)
            .emit_const(7i64)
            .emit_op(Op::iconst)
            .emit_const(5i64)
            .emit_op(Op::isub)
            .emit_op(Op::iret)
            .build();
        let executable = Executable::from(Function::new(code));
        let mut vm = VM::new(executable);
        let ret = vm.run()?;
        assert_eq!(ret, Val::Prm(2));
        Ok(())
    }

    #[test]
    fn vm_array() -> VMResult<()> {
        let code = CodeBuilder::default()
            .emit_array(Type::U, 4)
            .emit_op(Op::ret)
            .build();
        let mut vm = VM::new(Function::new(code).into());
        let _ret = vm.run()?;
        // dbg!(vm.gc);
        // assert_eq!(ret, Val::Prim(2));
        Ok(())
    }

    #[test]
    fn vm_array_load_store() -> VMResult<()> {
        // let xs = [0u8; 4];
        // xs[3] = 5;
        // xs[1] = 2;
        // xs[3] - xs[1]
        let code = CodeBuilder::default()
            .emit_array(Type::U, 4)
            .emit_loadl(0)
            .emit_iastore(3, 5)
            .emit_loadl(1)
            .emit_iastore(1, 2)
            .emit_loadl(0)
            .emit_iaload(3)
            .emit_loadl(0)
            .emit_iaload(1)
            .emit_op(Op::isub)
            .emit_op(Op::iret)
            .build();

        let mut vm = VM::new(Function::new(code).into());
        let ret = vm.run()?;
        assert_eq!(ret, Val::Prm(3));
        Ok(())
    }

    #[test]
    #[cfg(debug_assertions)]
    fn gc_release_unused_array() -> VMResult<()> {
        let code = CodeBuilder::default()
            .emit_array(Type::U, 4)
            .emit_iastore(0, 8)
            // when the second array is allocated the first should be freed as there are no
            // references to it
            .emit_array(Type::U, 8)
            .emit_op(Op::ret)
            .build();
        let mut vm = VM::new(Function::new(code).into());
        vm.run()?;
        // assert that the first array that was allocated is now freed
        assert!(vm.heap.gc.dbg_allocations[2].is_none());
        // assert that the first thing that was allocated is NOT freed
        assert!(vm.heap.gc.dbg_allocations[3].is_some());
        Ok(())
    }

    #[test]
    #[cfg(debug_assertions)]
    fn gc_maintain_arrays() -> VMResult<()> {
        let code = CodeBuilder::default()
            .emit_array(Type::U, 4)
            .emit_array(Type::U, 8)
            .emit_op(Op::ret)
            .build();
        let mut vm = VM::new(Function::new(code).into());
        vm.run()?;
        // println!("{:?}", vm.heap.gc);
        assert!(vm.heap.gc.dbg_allocations[0].is_some());
        assert!(vm.heap.gc.dbg_allocations[1].is_some());
        Ok(())
    }

    #[test]
    #[cfg(debug_assertions)]
    fn gc_release_multiple_unused_arrays() -> VMResult<()> {
        let code = CodeBuilder::default()
            .emit_array(Type::U, 4)
            .emit_array(Type::U, 8)
            .emit_iastore(0, 3)
            .emit_array(Type::U, 8)
            .emit_iastore(0, 5)
            .emit_array(Type::U, 8)
            .emit_op(Op::ret)
            .build();

        let mut vm = VM::new(Function::new(code).into());
        vm.run()?;
        assert!(vm.heap.gc.dbg_allocations[2].is_some());
        assert!(vm.heap.gc.dbg_allocations[3].is_none());
        assert!(vm.heap.gc.dbg_allocations[4].is_none());
        assert!(vm.heap.gc.dbg_allocations[5].is_some());
        Ok(())
    }

    #[test]
    #[cfg(debug_assertions)]
    fn gc_release_fn() -> VMResult<()> {
        Ok(())
    }
}
