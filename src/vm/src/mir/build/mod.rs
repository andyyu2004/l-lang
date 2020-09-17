mod cfg;
mod expr;
mod pat;
mod stmt;

use crate::ir;
use crate::mir::{self, *};
use crate::tir::{self, TirCtx};
use crate::ty::VariantTy;
use crate::typeck::TyCtx;
use cfg::Cfg;
use expr::LvalueBuilder;
use indexed_vec::{Idx, IndexVec};
use itertools::Itertools;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

const ENTRY_BLOCK: BlockId = BlockId(0);

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
    ctx: &'a TirCtx<'a, 'tcx>,
    body: &'tcx tir::Body<'tcx>,
) -> mir::Body<'tcx> {
    let mut builder = Builder::new(ctx, body);
    let _ = builder.build_body(ENTRY_BLOCK, body);
    let mir = builder.complete();
    mir::validate(&mir, &ctx);
    eprintln!("{}", mir);
    mir
}

/// a bit of a hacky way to generate the mir for variant constructors
/// `ty` should be the type of the enum adt
pub fn build_enum_ctors<'tcx>(tcx: TyCtx<'tcx>, item: &ir::Item) -> SmallVec<[mir::Item<'tcx>; 2]> {
    // todo deal with generics
    let scheme = tcx.collected_ty(item.id.def);
    let (_forall, ty) = scheme.expect_scheme();
    let (adt_ty, _) = ty.expect_adt();
    let mut vec = smallvec![];
    for (idx, variant) in adt_ty.variants.iter_enumerated() {
        let body = build_variant_ctor(tcx, ty, idx, variant);
        match body {
            None => continue,
            Some(body) => {
                eprintln!("{}", body);
                let kind = mir::ItemKind::Fn(body);
                let item = mir::Item {
                    span: item.span,
                    vis: item.vis,
                    ident: variant.ident,
                    id: variant.ctor.unwrap(),
                    kind,
                };
                vec.push(item);
            }
        }
    }
    vec
}

/// constructs the mir for a single variant constructor (if it is a function)
fn build_variant_ctor<'tcx>(
    tcx: TyCtx<'tcx>,
    ty: Ty<'tcx>,
    variant_idx: VariantIdx,
    variant: &VariantTy<'tcx>,
) -> Option<mir::Body<'tcx>> {
    // don't construct any mir for a constructor that is not a function
    if !variant.ctor_kind.is_function() {
        return None;
    }

    let ctor = variant.ctor.unwrap();

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
        .map(Operand::Ref)
        .collect_vec();

    let rvalue = Rvalue::Adt { adt, variant_idx, substs, fields };
    cfg.push_assignment(info, ENTRY_BLOCK, lvalue, rvalue);
    cfg.terminate(info, ENTRY_BLOCK, TerminatorKind::Return);
    Some(mir::Body { basic_blocks: cfg.basic_blocks, vars, argc: variant.fields.len() })
}

impl<'a, 'tcx> Builder<'a, 'tcx> {
    fn new(ctx: &'a TirCtx<'a, 'tcx>, body: &tir::Body<'tcx>) -> Self {
        let tcx = ctx.tcx;
        let mut builder = Self {
            tcx: ctx.tcx,
            ctx,
            cfg: Default::default(),
            vars: Default::default(),
            var_ir_map: Default::default(),
            argc: body.params.len(),
        };
        let info = builder.span_info(body.expr.span);
        builder.alloc_var(info, VarKind::Ret, builder.ctx.node_type(body.expr.id));
        builder
    }

    fn complete(self) -> mir::Body<'tcx> {
        mir::Body { basic_blocks: self.cfg.basic_blocks, vars: self.vars, argc: self.argc }
    }

    fn build_body(&mut self, mut block: BlockId, body: &'tcx tir::Body<'tcx>) -> BlockAnd<()> {
        let info = self.span_info(body.expr.span.hi());
        for param in body.params {
            let lvalue = self.alloc_arg(param.pat).into();
            set!(block = self.bind_pat_to_lvalue(block, param.pat, lvalue));
        }
        set!(block = self.write_expr(block, Lvalue::ret(), body.expr));
        self.terminate(info, block, TerminatorKind::Return);
        block.unit()
    }
}

struct Builder<'a, 'tcx> {
    tcx: TyCtx<'tcx>,
    ctx: &'a TirCtx<'a, 'tcx>,
    cfg: Cfg<'tcx>,
    var_ir_map: FxHashMap<ir::Id, VarId>,
    vars: IndexVec<VarId, Var<'tcx>>,
    argc: usize,
}

impl<'a, 'tcx> Builder<'a, 'tcx> {
    pub fn span_info(&self, span: Span) -> SpanInfo {
        SpanInfo { span }
    }

    fn ret_lvalue(&mut self) -> Lvalue<'tcx> {
        Lvalue::new(VarId::new(RETURN))
    }

    fn alloc_tmp(&mut self, info: SpanInfo, ty: Ty<'tcx>) -> VarId {
        self.alloc_var(info, VarKind::Tmp, ty)
    }

    /// create variable that has a corresponding var in the `ir`
    fn alloc_ir_var(&mut self, pat: &tir::Pattern<'tcx>, kind: VarKind) -> VarId {
        let info = self.span_info(pat.span);
        let idx = self.alloc_var(info, kind, pat.ty);
        self.var_ir_map.insert(pat.id, idx);
        idx
    }

    fn alloc_arg(&mut self, pat: &tir::Pattern<'tcx>) -> VarId {
        self.alloc_ir_var(pat, VarKind::Arg)
    }

    fn alloc_local(&mut self, pat: &tir::Pattern<'tcx>) -> VarId {
        self.alloc_ir_var(pat, VarKind::Local)
    }

    fn alloc_var(&mut self, info: SpanInfo, kind: VarKind, ty: Ty<'tcx>) -> VarId {
        // make it mutable by default, this can be unset later
        let var = Var { mtbl: Mutability::Mut, info, kind, ty };
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
