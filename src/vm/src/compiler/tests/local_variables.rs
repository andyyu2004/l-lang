use super::compile;
use crate::compiler::ConstId;
use crate::exec::{CodeBuilder, Op};
use indexed_vec::Idx;

#[test]
fn compile_local_var_check_popscp() {
    let executable = compile!("fn main() { let x = 0; let y = 1; }");
    let main = executable.constants[ConstId::new(0)].clone().as_fn();
    let code = CodeBuilder::default()
        .emit_dconst(0.0)
        .emit_dconst(1.0)
        .emit_op(Op::unit)
        .emit_popscp(2)
        .emit_op(Op::ret)
        .build();
    assert_eq!(code, main.code);
}

#[test]
fn compile_local_var_check_indices() {
    let executable = compile!("fn main() { let x = 0; let y = 1; y; x; }");
    let main = executable.constants[ConstId::new(0)].clone().as_fn();
    let code = CodeBuilder::default()
        .emit_dconst(0.0)
        .emit_dconst(1.0)
        .emit_loadl(1)
        .emit_op(Op::pop)
        .emit_loadl(0)
        .emit_op(Op::pop)
        .emit_op(Op::unit)
        .emit_popscp(2)
        .emit_op(Op::ret)
        .build();
    assert_eq!(code, main.code);
}

#[test]
fn compile_local_var_check_indices_multiple_scopes() {
}
