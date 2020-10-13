mod cfg;
mod ctx;
mod expr;
mod pat;
mod scope;
mod stmt;

use crate::set;
use ast::{Ident, Mutability};
use cfg::Cfg;
pub use ctx::MirCtx;
use index::{Idx, IndexVec};
use ir::{DefId, VariantIdx};
use itertools::Itertools;
use lcore::mir::*;
use lcore::ty::{Ty, TyCtx, VariantTy};
use rustc_hash::FxHashMap;
use scope::{ReleaseInfo, Scopes};
use span::Span;
use typeck::Typeof;

/// lowers `tir::Body` into `mir::Body`
pub fn build_fn<'a, 'tcx>(ctx: &'a MirCtx<'a, 'tcx>, body: tir::Body<'tcx>) -> Mir<'tcx> {
    let mut builder = Builder::new(ctx, &body);
    let _ = builder.build_body();
    let mir = builder.complete();
    // mir::analyse(&mir, &ctx);
    // mir::validate(&mir, &ctx);
    // eprintln!("{}", mir);
    mir
}

// a bit of a hacky way to generate the mir for variant constructors
/// `ty` should be the type of the enum adt
pub fn build_enum_ctors<'tcx>(
    tcx: TyCtx<'tcx>,
    item: &ir::Item,
) -> FxHashMap<DefId, (Ident, &'tcx Mir<'tcx>)> {
    // TODO deal with generics
    let scheme = tcx.collected_ty(item.id.def);
    let (_forall, ty) = scheme.expect_scheme();
    let (adt_ty, _) = ty.expect_adt();
    let mut map = FxHashMap::default();
    for (idx, variant) in adt_ty.variants.iter_enumerated() {
        let body = build_variant_ctor(tcx, ty, idx, variant);
        match body {
            None => continue,
            Some(body) => {
                // eprintln!("{}", body);
                let value = (item.ident.concat_as_path(variant.ident), body);
                let ctor_id = variant.ctor.unwrap();
                map.insert(ctor_id, value);
            }
        }
    }
    map
}

/// constructs the mir for a single variant constructor (if it is a function)
fn build_variant_ctor<'tcx>(
    tcx: TyCtx<'tcx>,
    ty: Ty<'tcx>,
    variant_idx: VariantIdx,
    variant: &VariantTy<'tcx>,
) -> Option<&'tcx Mir<'tcx>> {
    // don't construct any mir for a constructor that is not a function
    if !variant.ctor_kind.is_function() {
        return None;
    }

    // TODO get a proper span
    let info = SpanInfo { span: Span::empty() };
    let (adt, substs) = ty.expect_adt();

    let mut vars = IndexVec::<VarId, Var<'tcx>>::default();
    let mut alloc_var = |info: SpanInfo, kind: VarKind, ty: Ty<'tcx>| {
        let var = Var { mtbl: Mutability::Imm, info, kind, ty };
        vars.push(var)
    };

    let mut cfg = Cfg::default();
    let lvalue = alloc_var(info, VarKind::Ret, ty).into();

    // the `fields` of the variant are essentially the parameters of the constructor function
    let fields = variant
        .fields
        .iter()
        .map(|param| alloc_var(info, VarKind::Arg, param.ty(tcx, substs)))
        .map(Lvalue::new)
        .map(Operand::Lvalue)
        .collect_vec();

    let rvalue = Rvalue::Adt { adt, variant_idx, substs, fields };
    cfg.push_assignment(info, ENTRY_BLOCK, lvalue, rvalue);
    cfg.terminate(info, ENTRY_BLOCK, TerminatorKind::Return);
    let body = Mir { basic_blocks: cfg.basic_blocks, vars, argc: variant.fields.len() };
    Some(tcx.alloc(body))
}

impl<'a, 'tcx> Builder<'a, 'tcx> {
    fn new(ctx: &'a MirCtx<'a, 'tcx>, body: &'a tir::Body<'tcx>) -> Self {
        let tcx = ctx.tcx;
        let body_ty = body.expr.ty;
        let span = body.expr.span;
        let mut builder = Self {
            argc: body.params.len(),
            scopes: Default::default(),
            cfg: Default::default(),
            vars: Default::default(),
            var_ir_map: Default::default(),
            tcx,
            ctx,
            body,
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
        let info = self.span_info(self.body.expr.span.hi());
        self.with_scope(info, |this| {
            for param in &this.body.params {
                let box tir::Pattern { id, span, ty, .. } = param.pat;
                let lvalue = this.alloc_arg(id, span, ty).into();
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
    ctx: &'a MirCtx<'a, 'tcx>,
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
        let idx = self.alloc_var(info, kind, ty);
        self.var_ir_map.insert(id, idx);
        idx
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

impl BlockAndExt for BlockId {
    fn and<T>(self, v: T) -> BlockAnd<T> {
        BlockAnd(self, v)
    }

    fn unit(self) -> BlockAnd<()> {
        BlockAnd(self, ())
    }
}
