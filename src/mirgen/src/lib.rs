#![feature(decl_macro)]
#![feature(crate_visibility_modifier)]
#![feature(once_cell)]
#![feature(box_patterns, box_syntax)]

mod cfg;
mod ctor;
mod expr;
mod lowering_ctx;
mod pat;
mod scope;
mod stmt;

#[macro_use]
extern crate log;

#[macro_use]
extern crate smallvec;

use cfg::Cfg;
pub use ctor::build_variant_ctor;
pub use lowering_ctx::LoweringCtx;

use ast::Mutability;
use error::{LError, LResult};
use index::{Idx, IndexVec};
use ir::{DefId, DefNode, FnVisitor, ItemVisitor};
use lcore::queries::Queries;
use lcore::ty::{Instance, InstanceKind, TyCtx};
use lcore::{mir::*, ty::Ty};
use rustc_hash::FxHashMap;
use scope::{BreakType, ReleaseInfo, Scopes};
use span::Span;
use std::collections::BTreeMap;
use std::io::Write;

pub fn provide(queries: &mut Queries) {
    pat::provide(queries);
    *queries = Queries { mir_of, instance_mir, ..*queries }
}

macro halt_on_error($tcx:expr) {{
    if $tcx.sess.has_errors() {
        return Err(LError::ErrorReported);
    }
}}

fn instance_mir<'tcx>(tcx: TyCtx<'tcx>, instance: Instance<'tcx>) -> &'tcx Mir<'tcx> {
    match instance.kind {
        InstanceKind::Item => tcx.mir_of(instance.def_id),
        InstanceKind::Intrinsic => unreachable!("intrinsics don't have mir"),
    }
}

fn mir_of<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId) -> &'tcx Mir<'tcx> {
    let node = tcx.defs().get(def_id);
    match node {
        DefNode::Ctor(variant) => self::build_variant_ctor(tcx, variant),
        DefNode::Item(item) => match item.kind {
            ir::ItemKind::Fn(_, _, body) => self::build_mir(tcx, def_id, body),
            _ => panic!(),
        },
        DefNode::ImplItem(item) => match item.kind {
            ir::ImplItemKind::Fn(_, body) => self::build_mir(tcx, def_id, body),
        },
        DefNode::Field(..)
        | DefNode::ForeignItem(..)
        | DefNode::Variant(..)
        | DefNode::TyParam(..) => panic!(),
    }
}

fn build_mir<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId, body: &'tcx ir::Body<'tcx>) -> &'tcx Mir<'tcx> {
    with_lowering_ctx(tcx, def_id, |mut lctx| lctx.build_mir(body))
        .unwrap_or_else(|| tcx.alloc(Mir::default()))
}

fn with_lowering_ctx<'tcx, R>(
    tcx: TyCtx<'tcx>,
    def_id: DefId,
    f: impl FnOnce(LoweringCtx<'tcx>) -> R,
) -> Option<R> {
    // we return `None` if any errors occur during typecheck
    let tables = tcx.sess.try_run(|| tcx.typeck(def_id)).ok()?;
    let lctx = LoweringCtx::new(tcx, tables);
    Some(f(lctx))
}

// used only in old tests
pub fn build_tir<'tcx>(tcx: TyCtx<'tcx>) -> LResult<tir::Prog<'tcx>> {
    let prog = tcx.ir;
    let mut items = BTreeMap::new();

    for item in prog.items.values() {
        match item.kind {
            ir::ItemKind::Fn(..) => {
                if let Some(tir) =
                    with_lowering_ctx(tcx, item.id.def, |mut lctx| lctx.lower_item_tir(item))
                {
                    items.insert(item.id, tir);
                }
            }
            ir::ItemKind::Extern(_) => todo!(),
            // note that no tir is generated for enum constructors
            // the constructor code is generated at mir level only
            ir::ItemKind::TypeAlias(..) | ir::ItemKind::Enum(..) | ir::ItemKind::Struct(..) => {}
            ir::ItemKind::Mod(..) | ir::ItemKind::Use(..) | ir::ItemKind::Impl { .. } =>
                unreachable!(),
        }
    }
    halt_on_error!(tcx);
    Ok(tir::Prog { items })
}

pub fn dump_mir<'tcx>(tcx: TyCtx<'tcx>, writer: &mut dyn Write) {
    let mut mir_dump = MirDump { tcx, writer };
    mir_dump.visit_ir(tcx.ir);
}

struct MirDump<'a, 'tcx> {
    tcx: TyCtx<'tcx>,
    writer: &'a mut dyn Write,
}

impl<'a, 'tcx> FnVisitor<'tcx> for MirDump<'a, 'tcx> {
    fn visit_fn(&mut self, def_id: DefId) {
        let body = self.tcx.defs().body(def_id);
        let _ = with_lowering_ctx(self.tcx, def_id, |mut lctx| {
            let mir = lctx.build_mir(body);
            write!(self.writer, "\n{}", mir).unwrap();
        });
    }
}

/// lowers `tir::Body` into `mir::Body`
pub fn build_fn<'a, 'tcx>(ctx: &'a LoweringCtx<'tcx>, body: tir::Body<'tcx>) -> &'tcx Mir<'tcx> {
    let tcx = ctx.tcx;
    let mut builder = MirBuilder::new(ctx, &body);
    let _ = builder.build_body();
    let mir = ctx.alloc(builder.complete());

    mir::early_opt(tcx, mir);
    mir::typecheck(tcx, mir);
    mir::analyze(tcx, mir);
    mir::late_opt(tcx, mir);

    // eprintln!("{}", mir);
    mir
}

impl<'a, 'tcx> MirBuilder<'a, 'tcx> {
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

struct MirBuilder<'a, 'tcx> {
    tcx: TyCtx<'tcx>,
    ctx: &'a LoweringCtx<'tcx>,
    body: &'a tir::Body<'tcx>,
    scopes: Scopes<'tcx>,
    cfg: Cfg<'tcx>,
    vars: IndexVec<VarId, Var<'tcx>>,
    var_ir_map: FxHashMap<ir::Id, VarId>,
    argc: usize,
}

impl<'a, 'tcx> MirBuilder<'a, 'tcx> {
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

impl<'tcx> LvalueTy<'tcx> for MirBuilder<'_, 'tcx> {
    fn locals(&self) -> &IndexVec<VarId, Var<'tcx>> {
        &self.vars
    }
}

#[must_use]
struct BlockAnd<T> {
    block: BlockId,
    t: T,
}

/// set a block pointer and return the value
/// `let x = set!(block = self.foo(block, foo))`
#[macro_export]
macro_rules! set {
    ($x:ident = $c:expr) => {{
        let BlockAnd { block, t } = $c;
        $x = block;
        t
    }};

    ($c:expr) => {{
        let BlockAnd { block, t: () } = $c;
        block
    }};
}

trait BlockAndExt {
    fn and<T>(self, v: T) -> BlockAnd<T>;
    fn unit(self) -> BlockAnd<()>;
}

impl BlockAndExt for BlockId {
    fn and<T>(self, t: T) -> BlockAnd<T> {
        BlockAnd { block: self, t }
    }

    fn unit(self) -> BlockAnd<()> {
        BlockAnd { block: self, t: () }
    }
}
