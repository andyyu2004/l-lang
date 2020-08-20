use crate::mir::{self, *};
use crate::tir::{self, IrLoweringCtx};
use indexed_vec::{Idx, IndexVec};

mod expr;

/// lowers `tir::Body` into `mir::Body`
pub fn build_fn<'a, 'tcx>(
    ctx: IrLoweringCtx<'a, 'tcx>,
    body: &'tcx tir::Body<'tcx>,
) -> mir::Body<'tcx> {
    let mut builder = Builder::new(ctx);
    let init_block = BlockId::new(0);
    builder.build_body(init_block, body);
    todo!();
}

impl<'a, 'tcx> Builder<'a, 'tcx> {
    fn build_body(&mut self, mut block: BlockId, body: &'tcx tir::Body<'tcx>) -> mir::Body<'tcx> {
        todo!()
    }

    /// compiles `expr` into `dest`
    fn build_expr(
        &mut self,
        mut block: BlockId,
        expr: &'tcx tir::Expr<'tcx>,
        dest: Rvalue<'tcx>,
    ) -> BlockAnd<()> {
        match expr.kind {
            tir::ExprKind::Const(_) => {}
            tir::ExprKind::Bin(_, _, _) => {}
            tir::ExprKind::Unary(_, _) => {}
            tir::ExprKind::Block(_) => {}
            tir::ExprKind::VarRef(_) => {}
            tir::ExprKind::ItemRef(_) => {}
            tir::ExprKind::Tuple(_) => {}
            tir::ExprKind::Lambda(_) => {}
            tir::ExprKind::Call(_, _) => {}
            tir::ExprKind::Match(_, _) => {}
            tir::ExprKind::Assign(_, _) => {}
        };
        todo!()
    }

    fn build_rvalue(
        &mut self,
        mut block: BlockId,
        expr: &'tcx tir::Expr<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        match expr.kind {
            tir::ExprKind::Const(_) => {}
            tir::ExprKind::Bin(op, l, r) => {}
            tir::ExprKind::Unary(_, _) => {}
            tir::ExprKind::Block(_) => {}
            tir::ExprKind::VarRef(_) => {}
            tir::ExprKind::ItemRef(_) => {}
            tir::ExprKind::Tuple(_) => {}
            tir::ExprKind::Lambda(_) => {}
            tir::ExprKind::Call(_, _) => {}
            tir::ExprKind::Match(_, _) => {}
            tir::ExprKind::Assign(_, _) => {}
        }
        todo!()
    }
}

struct Builder<'a, 'tcx> {
    ctx: IrLoweringCtx<'a, 'tcx>,
    cfg: Cfg<'tcx>,
}

impl<'a, 'tcx> Builder<'a, 'tcx> {
    fn new(ctx: IrLoweringCtx<'a, 'tcx>) -> Self {
        Self { ctx, cfg: Default::default() }
    }
}

/// control flow graph
#[derive(Default)]
struct Cfg<'tcx> {
    basic_blocks: IndexVec<BlockId, BasicBlock<'tcx>>,
}

#[must_use]
struct BlockAnd<T>(mir::BlockId, T);

trait BlockAndExtension {
    fn and<T>(self, v: T) -> BlockAnd<T>;
    fn unit(self) -> BlockAnd<()>;
}

trait BlockAndExt {
    fn and<T>(self, v: T) -> BlockAnd<T>;
    fn unit(self) -> BlockAnd<()>;
}

impl BlockAndExt for mir::BlockId {
    fn and<T>(self, v: T) -> BlockAnd<T> {
        BlockAnd(self, v)
    }

    fn unit(self) -> BlockAnd<()> {
        BlockAnd(self, ())
    }
}

/// update a block pointer and return the value
/// `let x = unpack!(block = self.foo(block, foo))`
macro_rules! unpack {
    ($x:ident = $c:expr) => {{
        let BlockAnd(b, v) = $c;
        $x = b;
        v
    }};

    ($c:expr) => {{
        let BlockAnd(b, ()) = $c;
        b
    }};
}
