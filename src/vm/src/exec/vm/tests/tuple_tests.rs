use crate::error::VMResult;
use crate::{compiler::Executable, exec::*};

#[test]
fn tuple_simple() -> VMResult<()> {
    let main =
        CodeBuilder::default().emit_iconst(2).emit_iconst(3).emit_tuple(2).emit_op(Op::ret).build();

    let exec = Executable::from(Function::new(main));
    let mut vm = VM::with_default_gc(exec);
    let ret = vm.run()?;
    assert_eq!(format!("{}", ret), "(2,3)");

    Ok(())
}

#[test]
fn tuple_nested() -> VMResult<()> {
    let main = CodeBuilder::default()
        .emit_iconst(3)
        .emit_iconst(7)
        .emit_iconst(8)
        .emit_iconst(9)
        .emit_tuple(3)
        .emit_tuple(2)
        .emit_op(Op::ret)
        .build();

    let exec = Executable::from(Function::new(main));
    let mut vm = VM::with_default_gc(exec);
    let ret = vm.run()?;
    assert_eq!(format!("{}", ret), "(3,(7,8,9))");
    Ok(())
}
