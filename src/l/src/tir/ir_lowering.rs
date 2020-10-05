use crate::ast::{Ident, Lit, Mutability, UnaryOp, Visibility};
use crate::ir::{self, CtorKind, DefId, DefKind, FieldIdx, VariantIdx};
use crate::lexer::Symbol;
use crate::mir;
use crate::tir;
use crate::ty::*;
use crate::typeck::inference::InferCtx;
use crate::typeck::{TyCtx, TypeckTables};
use indexed_vec::Idx;
use smallvec::SmallVec;
use std::marker::PhantomData;
use std::ops::Deref;

/// ir -> tir -> mir
pub struct TirCtx<'a, 'tcx> {
    tcx: TyCtx<'tcx>,
    infcx: &'a InferCtx<'a, 'tcx>,
    tables: &'a TypeckTables<'tcx>,
}

impl<'a, 'tcx> Deref for TirCtx<'a, 'tcx> {
    type Target = &'a InferCtx<'a, 'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.infcx
    }
}

impl<'a, 'tcx> TirCtx<'a, 'tcx> {
    pub fn new(infcx: &'a InferCtx<'a, 'tcx>, tables: &'a TypeckTables<'tcx>) -> Self {
        Self { infcx, tcx: infcx.tcx, tables }
    }

    /// ir -> tir
    pub fn lower_item_tir(&mut self, item: &ir::Item<'tcx>) -> tir::Item<'tcx> {
        let tir = item.to_tir(self);
        tir
    }

    /// ir -> tir -> mir
    pub fn build_mir(&mut self, body: &ir::Body<'tcx>) -> &'tcx mir::Mir<'tcx> {
        let tir = body.to_tir(self);
        eprintln!("{}", tir);
        self.tcx.alloc(mir::build_fn(self, tir))
    }

    pub fn node_type(&self, id: ir::Id) -> Ty<'tcx> {
        info!("irloweringctx: query typeof {:?}", id);
        self.tables.node_type(id)
    }

    fn lower_tuple_subpats(&mut self, pats: &[ir::Pattern<'tcx>]) -> &'tcx [tir::FieldPat<'tcx>] {
        let tcx = self.tcx;
        let pats = pats.iter().enumerate().map(|(i, pat)| tir::FieldPat {
            field: ir::FieldIdx::new(i),
            pat: pat.to_tir_alloc(self),
        });
        tcx.alloc_tir_iter(pats)
    }
}

/// trait for conversion to tir
pub trait Tir<'tcx> {
    type Output;
    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output;

    fn to_tir_alloc(&self, ctx: &mut TirCtx<'_, 'tcx>) -> &'tcx Self::Output {
        let tir = self.to_tir(ctx);
        ctx.tcx.alloc_tir(tir)
    }
}

impl<'tcx> Tir<'tcx> for ir::Field<'tcx> {
    type Output = tir::Field<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        tir::Field {
            index: ctx.tables.field_index(self.id),
            expr: self.expr.to_tir_alloc(ctx),
            ident: self.ident,
        }
    }
}

impl<'tcx> Tir<'tcx> for ir::Item<'tcx> {
    type Output = tir::Item<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let &Self { span, id, ident, vis, ref kind } = self;
        match kind {
            ir::ItemKind::Fn(sig, generics, body) => {
                let ty = ctx.tcx.collected_ty(self.id.def);
                let kind = tir::ItemKind::Fn(ty, generics.to_tir(ctx), body.to_tir(ctx));
                tir::Item { kind, span, id, ident, vis }
            }
            ir::ItemKind::Impl { .. } => todo!(),
            ir::ItemKind::Enum(..) | ir::ItemKind::Struct(..) => unreachable!(),
        }
    }
}

impl<'tcx> Tir<'tcx> for ir::Param<'tcx> {
    type Output = tir::Param<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let &Self { id, span, ref pat } = self;
        tir::Param { id, span, pat: pat.to_tir_alloc(ctx) }
    }
}

impl<'tcx> Tir<'tcx> for ir::Pattern<'tcx> {
    type Output = tir::Pattern<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let &Self { id, span, kind } = self;
        let kind = match kind {
            ir::PatternKind::Wildcard => tir::PatternKind::Wildcard,
            ir::PatternKind::Binding(ident, sub, m) => {
                let subpat = sub.map(|pat| pat.to_tir_alloc(ctx));
                tir::PatternKind::Binding(m, ident, subpat)
            }
            ir::PatternKind::Tuple(pats) => tir::PatternKind::Field(ctx.lower_tuple_subpats(pats)),
            ir::PatternKind::Lit(expr) => tir::PatternKind::Lit(expr.to_tir_alloc(ctx)),
            ir::PatternKind::Variant(path, pats) => ctx.lower_variant_pat(self, path, pats),
            ir::PatternKind::Path(path) => ctx.lower_variant_pat(self, path, &[]),
        };
        let ty = ctx.node_type(id);
        tir::Pattern { id, span, kind, ty }
    }
}

impl<'tcx> TirCtx<'_, 'tcx> {
    fn lower_variant_pat(
        &mut self,
        pat: &ir::Pattern<'tcx>,
        path: &ir::Path<'tcx>,
        pats: &'tcx [ir::Pattern<'tcx>],
    ) -> tir::PatternKind<'tcx> {
        let ty = self.node_type(pat.id);
        let (adt, substs) = ty.expect_adt();
        let idx = adt.variant_idx_with_res(path.res);
        tir::PatternKind::Variant(adt, substs, idx, pats.to_tir(self))
    }

    // useful impl ideas
    // https://stackoverflow.com/questions/43171341/swift-function-object-wrapper-in-apple-swift
    fn lower_closure(&mut self, closure: &ir::Expr, body: &ir::Body<'tcx>) -> tir::ExprKind<'tcx> {
        let body = body.to_tir_alloc(self);
        let upvar_captures = self.tables.upvar_captures_for_closure(closure.id);
        let upvars = self
            .tcx
            .alloc_tir_iter(upvar_captures.iter().map(|&upvar| self.capture_upvar(closure, upvar)));
        tir::ExprKind::Closure { body, upvars }
    }

    /// manually constructs a (mutable?) borrow expression to an upvar
    fn capture_upvar(&mut self, closure: &ir::Expr, upvar: UpvarId) -> tir::Expr<'tcx> {
        let id = upvar.var_id;
        let span = closure.span;
        let ty = self.node_type(id);
        // rebuild the `VarRef` expressions that the upvar refers to
        let captured = self.alloc_tir(tir::Expr { span, ty, kind: tir::ExprKind::VarRef(id) });
        // construct a mutable borrow expression to the captured upvar
        let borrow_expr = tir::Expr {
            span,
            ty: self.mk_ptr_ty(Mutability::Mut, ty),
            kind: tir::ExprKind::Ref(captured),
        };
        borrow_expr
    }
}

impl<'tcx> Tir<'tcx> for ir::Body<'tcx> {
    type Output = &'tcx tir::Body<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let tcx = ctx.tcx;
        let params = self.params.to_tir(ctx);
        let body = tir::Body { params, expr: self.expr.to_tir_alloc(ctx) };
        ctx.tcx.alloc_tir(body)
    }
}

impl<'tcx> Tir<'tcx> for ir::Generics<'tcx> {
    type Output = &'tcx tir::Generics<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        ctx.tcx.alloc_tir(tir::Generics { data: 0, pd: PhantomData })
    }
}

impl<'tcx> Tir<'tcx> for ir::Let<'tcx> {
    type Output = tir::Let<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let tcx = ctx.tcx;
        tir::Let {
            id: self.id,
            pat: self.pat.to_tir_alloc(ctx),
            init: self.init.map(|init| init.to_tir_alloc(ctx)),
        }
    }
}

impl<'tcx> Tir<'tcx> for ir::Stmt<'tcx> {
    type Output = tir::Stmt<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let &Self { id, span, ref kind } = self;
        let kind = match kind {
            ir::StmtKind::Let(l) => tir::StmtKind::Let(l.to_tir_alloc(ctx)),
            // we can map both semi and expr to expressions and their distinction is no longer
            // important after typechecking is done
            ir::StmtKind::Expr(expr) | ir::StmtKind::Semi(expr) =>
                tir::StmtKind::Expr(expr.to_tir_alloc(ctx)),
        };
        tir::Stmt { id, span, kind }
    }
}

impl<'tcx> Tir<'tcx> for ir::Block<'tcx> {
    type Output = tir::Block<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let tcx = ctx.tcx;
        let stmts = tcx.alloc_tir_iter(self.stmts.iter().map(|stmt| stmt.to_tir(ctx)));
        let expr = self.expr.map(|expr| expr.to_tir_alloc(ctx));
        tir::Block { id: self.id, stmts, expr }
    }
}

impl<'tcx, T> Tir<'tcx> for Option<T>
where
    T: Tir<'tcx>,
{
    type Output = Option<T::Output>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        self.as_ref().map(|t| t.to_tir(ctx))
    }
}

impl<'tcx, T> Tir<'tcx> for &'tcx [T]
where
    T: Tir<'tcx>,
{
    type Output = &'tcx [T::Output];

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let tcx = ctx.tcx;
        tcx.alloc_tir_iter(self.iter().map(|t| t.to_tir(ctx)))
    }

    fn to_tir_alloc(&self, ctx: &mut TirCtx<'_, 'tcx>) -> &'tcx Self::Output {
        panic!("use `to_tir` for slices")
    }
}

impl<'tcx> TirCtx<'_, 'tcx> {
    fn lower_path(&self, expr: &ir::Expr<'tcx>, path: &ir::Path<'tcx>) -> tir::ExprKind<'tcx> {
        match path.res {
            ir::Res::Local(id) => tir::ExprKind::VarRef(id),
            ir::Res::Def(def_id, def_kind) => match def_kind {
                ir::DefKind::Ctor(CtorKind::Unit, ..) => {
                    let (adt, substs) = self.node_type(expr.id).expect_adt();
                    tir::ExprKind::Adt {
                        adt,
                        substs,
                        fields: &[],
                        variant_idx: adt.variant_idx_with_ctor(def_id),
                    }
                }
                // functions and function-like variant constructors
                ir::DefKind::Ctor(CtorKind::Tuple, ..) | ir::DefKind::Fn =>
                    tir::ExprKind::ItemRef(def_id),
                ir::DefKind::AssocFn => todo!(),
                ir::DefKind::Impl => todo!(),
                ir::DefKind::Ctor(..) => todo!(),
                ir::DefKind::TyParam(_) => todo!(),
                ir::DefKind::Enum => todo!(),
                ir::DefKind::Struct => todo!(),
            },
            ir::Res::SelfTy => todo!(),
            ir::Res::Err | ir::Res::PrimTy(_) => unreachable!(),
        }
    }
}

impl<'tcx> Tir<'tcx> for ir::Expr<'tcx> {
    type Output = tir::Expr<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        ctx.lower_expr(self)
    }
}

impl<'a, 'tcx> TirCtx<'a, 'tcx> {
    fn lower_expr_no_adjust(&mut self, expr: &ir::Expr<'tcx>) -> tir::Expr<'tcx> {
        let &ir::Expr { span, id, ref kind } = expr;
        let ty = self.node_type(expr.id);
        let kind = match kind {
            ir::ExprKind::Bin(op, l, r) =>
                tir::ExprKind::Bin(*op, l.to_tir_alloc(self), r.to_tir_alloc(self)),
            ir::ExprKind::Unary(UnaryOp::Deref, expr) =>
                tir::ExprKind::Deref(expr.to_tir_alloc(self)),
            ir::ExprKind::Unary(UnaryOp::Ref, expr) => tir::ExprKind::Ref(expr.to_tir_alloc(self)),
            ir::ExprKind::Unary(op, expr) => tir::ExprKind::Unary(*op, expr.to_tir_alloc(self)),
            ir::ExprKind::Block(block) => tir::ExprKind::Block(block.to_tir_alloc(self)),
            ir::ExprKind::Path(path) => self.lower_path(expr, path),
            ir::ExprKind::Tuple(xs) => tir::ExprKind::Tuple(xs.to_tir(self)),
            ir::ExprKind::Closure(_sig, body) => self.lower_closure(expr, body),
            ir::ExprKind::Call(f, args) =>
                tir::ExprKind::Call(f.to_tir_alloc(self), args.to_tir(self)),
            ir::ExprKind::Lit(lit) => tir::ExprKind::Const(lit.to_tir_alloc(self)),
            ir::ExprKind::Match(expr, arms, _) =>
                tir::ExprKind::Match(expr.to_tir_alloc(self), arms.to_tir(self)),
            ir::ExprKind::Struct(path, fields) => match ty.kind {
                TyKind::Adt(adt, substs) => match adt.kind {
                    AdtKind::Struct => tir::ExprKind::Adt {
                        adt,
                        substs,
                        variant_idx: VariantIdx::new(0),
                        fields: fields.to_tir(self),
                    },
                    AdtKind::Enum => {
                        let variant_idx = adt.variant_idx_with_res(path.res);
                        tir::ExprKind::Adt { adt, substs, variant_idx, fields: fields.to_tir(self) }
                    }
                },
                _ => unreachable!(),
            },
            ir::ExprKind::Ret(expr) => tir::ExprKind::Ret(expr.map(|expr| expr.to_tir_alloc(self))),
            ir::ExprKind::Assign(l, r) =>
                tir::ExprKind::Assign(l.to_tir_alloc(self), r.to_tir_alloc(self)),
            ir::ExprKind::Field(base, _) =>
                tir::ExprKind::Field(base.to_tir_alloc(self), self.tables.field_index(expr.id)),
            ir::ExprKind::Box(expr) => tir::ExprKind::Box(expr.to_tir_alloc(self)),
        };
        tir::Expr { span, kind, ty }
    }

    fn lower_expr(&mut self, expr: &ir::Expr<'tcx>) -> tir::Expr<'tcx> {
        let tir = self.lower_expr_no_adjust(expr);
        let adjustments = self.tables.adjustments_for_expr(expr);
        self.apply_adjustments(tir, adjustments)
    }

    fn apply_adjustments(
        &mut self,
        expr: tir::Expr<'tcx>,
        adjustments: &[Adjustment<'tcx>],
    ) -> tir::Expr<'tcx> {
        adjustments.iter().fold(expr, |expr, adj| self.apply_adjustment(expr, adj))
    }

    fn apply_adjustment(
        &mut self,
        expr: tir::Expr<'tcx>,
        adjustment: &Adjustment<'tcx>,
    ) -> tir::Expr<'tcx> {
        let span = expr.span;
        let kind = match adjustment.kind {
            AdjustmentKind::Deref => tir::ExprKind::Deref(self.alloc_tir(expr)),
            AdjustmentKind::NeverToAny => todo!(),
        };
        tir::Expr { ty: adjustment.ty, span, kind }
    }
}

impl<'tcx> Tir<'tcx> for Lit {
    type Output = Const<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        match *self {
            Lit::Float(n) => Const::new(ConstKind::Float(n), ctx.tcx.types.float),
            Lit::Bool(b) => Const::new(ConstKind::Bool(b), ctx.tcx.types.boolean),
            Lit::Int(i) => Const::new(ConstKind::Int(i), ctx.tcx.types.int),
        }
    }

    fn to_tir_alloc(&self, ctx: &mut TirCtx<'_, 'tcx>) -> &'tcx Self::Output {
        let c = self.to_tir(ctx);
        ctx.tcx.intern_const(c)
    }
}

impl<'tcx> Tir<'tcx> for ir::Arm<'tcx> {
    type Output = tir::Arm<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let &ir::Arm { id, span, ref pat, ref body, ref guard } = self;
        tir::Arm {
            id,
            span,
            pat: pat.to_tir_alloc(ctx),
            body: body.to_tir_alloc(ctx),
            guard: guard.map(|expr| expr.to_tir_alloc(ctx)),
        }
    }
}
