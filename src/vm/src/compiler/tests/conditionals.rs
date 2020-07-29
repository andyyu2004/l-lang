use super::compile_expr;
use crate::compiler::ConstId;
use crate::exec::{CodeBuilder, Op};
use indexed_vec::Idx;

/// approximate desugaring
/// match false {
///     true => 5,
///     _ => 4,
/// }
#[test]
fn compile_simple_if_stmt() {
    let executable = compile_expr!("if false { 5 } else { 4 };");
    let main = executable.constants[ConstId::new(0)].clone().as_fn();
    let code = CodeBuilder::default()
        // eval scrutinee expr `false`
        .emit_uconst(0)
        // compare scrutinee with first branch `true` (== 1)
        .emit_op(Op::dup)
        .emit_uconst(1)
        // if not matched, jump to next branch
        .emit_known_jmp(Op::jmpneq, 0x0c)
        // if matched, eval expr and jump after the match
        .emit_dconst(5.0)
        .emit_known_jmp(Op::jmp, 0x11)
        // otherwise eval other branch
        .emit_op(Op::dup)
        // the second dup is because the pattern is a wildcard and should always match
        .emit_op(Op::dup)
        .emit_known_jmp(Op::jmpneq, 0x0c)
        .emit_dconst(4.0)
        .emit_known_jmp(Op::jmp, 0)
        .emit_op(Op::pop)
        .emit_op(Op::unit)
        .emit_op(Op::ret)
        .build();
    assert_eq!(code, main.code);
}
