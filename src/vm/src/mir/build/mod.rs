mod cfg;
mod expr;
mod stmt;

use crate::mir::{self, *};
use crate::tir::{self, IrLoweringCtx};
use cfg::Cfg;
use indexed_vec::{Idx, IndexVec};

/// set a block pointer and return the value
/// `let x = set!(block = self.foo(block, foo))`
#[macro_export]
macro_rules! set {
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

/// lowers `tir::Body` into `mir::Body`
pub fn build_fn<'a, 'tcx>(
    ctx: IrLoweringCtx<'a, 'tcx>,
    body: &'tcx tir::Body<'tcx>,
) -> mir::Body<'tcx> {
    let mut builder = Builder::new(ctx);
    let entry_block = BlockId::new(0);
    let _ = builder.build_body(entry_block, body);
    let mir = builder.complete();
    dbg!(&mir);
    mir
}

impl<'a, 'tcx> Builder<'a, 'tcx> {
    fn complete(self) -> mir::Body<'tcx> {
        mir::Body { basic_blocks: self.cfg.basic_blocks }
    }

    fn build_body(&mut self, mut block: BlockId, body: &'tcx tir::Body<'tcx>) -> BlockAnd<()> {
        self.expr(block, body.expr, Lvalue::ret(body.expr.ty))
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

    pub fn span_info(&self, span: Span) -> SpanInfo {
        SpanInfo { span }
    }
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
