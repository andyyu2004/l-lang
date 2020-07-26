use super::compile;
use crate::compiler::ConstId;
use crate::exec::{CodeBuilder, Op};
use indexed_vec::Idx;

#[test]
fn simple_closure() {
    let src = r#"
    fn main() -> number {
        let x = 5;
        let f = fn(y) => x + y;
        apply(f, 8)
    }

    fn apply(f: fn(number) -> number, x: number) -> number {
        f(x)
    }"#;

    let executable = compile!(src);
    let main = executable.constants[ConstId::new(0)].clone().as_fn();
    let apply = &executable.constants[ConstId::new(1)].clone().as_fn();
    let f = &executable.constants[ConstId::new(2)].clone().as_fn();

    let main_code = CodeBuilder::default()
        .emit_dconst(5.0)
        .emit_closure(2, vec![(true, 0)]) // `x` is captured in enclosing and has local_idx 0
        .emit_ldc(1)
        .emit_loadl(1)
        .emit_dconst(8.0)
        .emit_invoke(2)
        // pop `x` and `f`
        .emit_popscp(2)
        .emit_op(Op::ret)
        .build();

    let apply_code = CodeBuilder::default()
        .emit_loadl(0)
        .emit_loadl(1)
        .emit_invoke(1)
        // pop the 2 parameters
        .emit_popscp(2)
        .emit_op(Op::ret)
        .build();

    let f_code = CodeBuilder::default()
        .emit_loadu(0)
        .emit_loadl(0)
        .emit_op(Op::dadd)
        // only pop the parameter `y` not the upvar `x`
        .emit_popscp(1)
        .emit_op(Op::ret)
        .build();

    assert_eq!(main.code, main_code);
    assert_eq!(apply.code, apply_code);
    assert_eq!(f_code, f.code);
}
