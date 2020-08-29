mod cfg;
mod expr;
mod pat;
mod stmt;

use crate::ir;
use crate::mir::{self, *};
use crate::tir::{self, IrLoweringCtx};
use cfg::Cfg;
use indexed_vec::{Idx, IndexVec};
use rustc_hash::FxHashMap;

static ENTRY_BLOCK_ID: usize = 0;

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
    let mut builder = Builder::new(ctx, body);
    let entry_block = BlockId::new(ENTRY_BLOCK_ID);
    let _ = builder.build_body(entry_block, body);
    let mir = builder.complete();
    println!("{}", mir);
    mir
}

impl<'a, 'tcx> Builder<'a, 'tcx> {
    fn new(ctx: IrLoweringCtx<'a, 'tcx>, body: &tir::Body<'tcx>) -> Self {
        let mut cfg = Cfg::default();
        let tcx = ctx.tcx;
        assert_eq!(cfg.append_basic_block().index(), ENTRY_BLOCK_ID);
        let vars = IndexVec::default();
        let mut builder = Self { ctx, cfg, vars, var_ir_map: Default::default() };
        let info = builder.span_info(body.expr.span);
        builder.alloc_var(info, VarKind::Ret, builder.ctx.node_type(body.expr.id));
        builder
    }

    fn complete(self) -> mir::Body<'tcx> {
        mir::Body { basic_blocks: self.cfg.basic_blocks, vars: self.vars }
    }

    fn build_body(&mut self, mut block: BlockId, body: &'tcx tir::Body<'tcx>) -> BlockAnd<()> {
        set!(block = self.write_expr(block, Lvalue::ret(), body.expr));
        let info = self.span_info(body.expr.span.hi());
        self.cfg.terminate(info, block, TerminatorKind::Return);
        block.unit()
    }
}

struct Builder<'a, 'tcx> {
    ctx: IrLoweringCtx<'a, 'tcx>,
    cfg: Cfg<'tcx>,
    var_ir_map: FxHashMap<ir::Id, VarId>,
    vars: IndexVec<VarId, Var<'tcx>>,
}

impl<'a, 'tcx> Builder<'a, 'tcx> {
    pub fn span_info(&self, span: Span) -> SpanInfo {
        SpanInfo { span }
    }

    fn return_block(&mut self) -> BlockId {
        self.cfg.append_basic_block()
    }

    fn alloc_tmp(&mut self, info: SpanInfo, ty: Ty<'tcx>) -> VarId {
        self.alloc_var(info, VarKind::Tmp, ty)
    }

    fn alloc_local(&mut self, pat: &tir::Pattern<'tcx>) -> VarId {
        let info = self.span_info(pat.span);
        let var = Var { info, kind: VarKind::Local, ty: pat.ty };
        let idx = self.vars.push(var);
        self.var_ir_map.insert(pat.id, idx);
        idx
    }

    fn alloc_var(&mut self, info: SpanInfo, kind: VarKind, ty: Ty<'tcx>) -> VarId {
        let var = Var { info, kind, ty };
        self.vars.push(var)
    }
}

#[must_use]
struct BlockAnd<T>(mir::BlockId, T);

impl<T> BlockAnd<T> {
    fn unpack(self) -> (BlockId, T) {
        let Self(block, t) = self;
        (block, t)
    }
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
