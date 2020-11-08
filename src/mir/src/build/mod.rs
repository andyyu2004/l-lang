mod cfg;
mod ctor;
mod ctx;
mod expr;
mod pat;
mod scope;
mod stmt;

pub use ctor::build_variant_ctor;
pub use ctx::LoweringCtx;

use crate::set;
use ast::Mutability;
use cfg::Cfg;
use index::{Idx, IndexVec};
use lcore::mir::*;
use lcore::ty::{Ty, TyCtx};
use rustc_hash::FxHashMap;
use scope::{ReleaseInfo, Scopes};
use span::Span;

/// lowers `tir::Body` into `mir::Body`
pub fn build_fn<'a, 'tcx>(ctx: &'a LoweringCtx<'tcx>, body: tir::Body<'tcx>) -> &'tcx Mir<'tcx> {
    let mut builder = Builder::new(ctx, &body);
    let _ = builder.build_body();
    let mir = ctx.alloc(builder.complete());
    // crate::analyse(&mir, &ctx);
    crate::typecheck(ctx.tcx, &mir);
    // eprintln!("{}", mir);
    mir
}

impl<'a, 'tcx> Builder<'a, 'tcx> {
    fn new(ctx: &'a LoweringCtx<'tcx>, body: &'a tir::Body<'tcx>) -> Self {
        let tcx = ctx.tcx;
        let body_ty = body.expr.ty;
        let span = body.expr.span;
        let mut builder = Self {
            tcx,
            ctx,
            body,
            argc: body.params.len(),
            scopes: Default::default(),
            cfg: Default::default(),
            vars: Default::default(),
            var_ir_map: Default::default(),
        };
        let info = builder.span_info(span);
        builder.alloc_var(info, VarKind::Ret, body_ty);
        builder
    }

    fn complete(self) -> Mir<'tcx> {
        Mir { basic_blocks: self.cfg.basic_blocks, vars: self.vars, argc: self.argc }
    }

    /// entry point to building
    fn build_body(&mut self) -> BlockAnd<()> {
        let mut block = ENTRY_BLOCK;
        let info = self.span_info(self.body.expr.span);
        self.with_scope(info, |this| {
            for param in &this.body.params {
                let box tir::Pattern { id, span, ty, .. } = param.pat;
                let lvalue = Lvalue::from(this.alloc_arg(id, span, ty));
                if let tir::PatternKind::Binding(..) = param.pat.kind {
                    // nothing meaningful to recursively bind to
                    continue;
                }
                set!(block = this.bind_pat_to_lvalue(block, &param.pat, lvalue));
            }
            set!(block = this.write_expr(block, Lvalue::ret(), &this.body.expr));
            this.terminate(info, block, TerminatorKind::Return);
            block.unit()
        })
    }
}

struct Builder<'a, 'tcx> {
    tcx: TyCtx<'tcx>,
    ctx: &'a LoweringCtx<'tcx>,
    body: &'a tir::Body<'tcx>,
    scopes: Scopes<'tcx>,
    cfg: Cfg<'tcx>,
    vars: IndexVec<VarId, Var<'tcx>>,
    var_ir_map: FxHashMap<ir::Id, VarId>,
    argc: usize,
}

impl<'a, 'tcx> Builder<'a, 'tcx> {
    pub fn span_info(&self, span: Span) -> SpanInfo {
        SpanInfo { span }
    }

    fn ret_lvalue(&mut self) -> Lvalue<'tcx> {
        Lvalue::new(RET_VAR)
    }

    fn alloc_tmp(&mut self, info: SpanInfo, ty: Ty<'tcx>) -> VarId {
        self.alloc_var(info, VarKind::Tmp, ty)
    }

    /// create variable that has a corresponding var in the `ir`
    fn alloc_ir_var(&mut self, id: ir::Id, span: Span, ty: Ty<'tcx>, kind: VarKind) -> VarId {
        let info = self.span_info(span);
        let var_id = self.alloc_var(info, kind, ty);
        let prev = self.var_ir_map.insert(id, var_id);
        if prev.is_some() {
            panic!("two mir vars allocated for id `{}`", id);
        }
        var_id
    }

    fn operand_ty(&self, operand: Operand<'tcx>) -> Ty<'tcx> {
        operand.ty(self.tcx, self)
    }

    fn lvalue_ty(&self, lvalue: Lvalue<'tcx>) -> Ty<'tcx> {
        lvalue.ty(self.tcx, self)
    }

    fn alloc_arg(&mut self, id: ir::Id, span: Span, ty: Ty<'tcx>) -> VarId {
        self.alloc_ir_var(id, span, ty, VarKind::Arg)
    }

    fn alloc_local(&mut self, id: ir::Id, span: Span, ty: Ty<'tcx>) -> VarId {
        self.alloc_ir_var(id, span, ty, VarKind::Local)
    }

    fn alloc_upvar(&mut self, id: ir::Id, span: Span, ty: Ty<'tcx>) -> VarId {
        self.alloc_ir_var(id, span, ty, VarKind::Upvar)
    }

    fn alloc_var(&mut self, info: SpanInfo, kind: VarKind, ty: Ty<'tcx>) -> VarId {
        // make it mutable by default, this can be unset later
        let var = Var { mtbl: Mutability::Mut, info, kind, ty };
        self.vars.push(var)
    }
}

impl<'tcx> LvalueTy<'tcx> for Builder<'_, 'tcx> {
    fn locals(&self) -> &IndexVec<VarId, Var<'tcx>> {
        &self.vars
    }
}

#[must_use]
struct BlockAnd<T>(BlockId, T);

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

trait BlockAndExt {
    fn and<T>(self, v: T) -> BlockAnd<T>;
    fn unit(self) -> BlockAnd<()>;
}

impl BlockAndExt for BlockId {
    fn and<T>(self, v: T) -> BlockAnd<T> {
        BlockAnd(self, v)
    }

    fn unit(self) -> BlockAnd<()> {
        BlockAnd(self, ())
    }
}
