use crate as tir;
use ast::{Lit, Mutability, UnaryOp};
use index::Idx;
use infer::InferCtx;
use ir::{self, CtorKind, DefKind, Res, VariantIdx};
use lcore::mir;
use lcore::ty::*;
use lcore::TyCtx;
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
        todo!()
        // self.tcx.alloc(mir::build_fn(self, tir))
    }

    pub fn node_type(&self, id: ir::Id) -> Ty<'tcx> {
        info!("irloweringctx: query typeof {:?}", id);
        self.tables.node_type(id)
    }

    fn lower_tuple_subpats(&mut self, pats: &[ir::Pattern<'tcx>]) -> Vec<tir::FieldPat<'tcx>> {
        let tcx = self.tcx;
        pats.iter()
            .enumerate()
            .map(|(i, pat)| tir::FieldPat {
                field: ir::FieldIdx::new(i),
                pat: box pat.to_tir(self),
            })
            .collect()
    }
}

/// trait for conversion to tir
pub trait Tir<'tcx> {
    type Output;
    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output;
}

impl<'tcx> Tir<'tcx> for ir::Field<'tcx> {
    type Output = tir::Field<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        tir::Field {
            index: ctx.tables.field_index(self.id),
            expr: box self.expr.to_tir(ctx),
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
                let kind = tir::ItemKind::Fn(ty, generics.to_tir(ctx), box body.to_tir(ctx));
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
        tir::Param { id, span, pat: box pat.to_tir(ctx) }
    }
}

impl<'tcx> Tir<'tcx> for ir::Pattern<'tcx> {
    type Output = tir::Pattern<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let &Self { id, span, kind } = self;
        let kind = match kind {
            ir::PatternKind::Wildcard => tir::PatternKind::Wildcard,
            ir::PatternKind::Binding(ident, sub, m) => {
                let subpat = sub.map(|pat| box pat.to_tir(ctx));
                tir::PatternKind::Binding(m, ident, subpat)
            }
            ir::PatternKind::Tuple(pats) => tir::PatternKind::Field(ctx.lower_tuple_subpats(pats)),
            ir::PatternKind::Lit(expr) => tir::PatternKind::Lit(box expr.to_tir(ctx)),
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
        let body = box body.to_tir(self);
        let upvar_captures = self.tables.upvar_captures_for_closure(closure.id);
        let upvars =
            upvar_captures.iter().map(|&upvar| self.capture_upvar(closure, upvar)).collect();
        tir::ExprKind::Closure { body, upvars }
    }

    /// manually constructs a (mutable?) borrow expression to an upvar
    fn capture_upvar(&mut self, closure: &ir::Expr, upvar: UpvarId) -> tir::Expr<'tcx> {
        let id = upvar.var_id;
        let span = closure.span;
        let ty = self.node_type(id);
        // rebuild the `VarRef` expressions that the upvar refers to
        let captured = box tir::Expr { span, ty, kind: tir::ExprKind::VarRef(id) };
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
    type Output = tir::Body<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let tcx = ctx.tcx;
        let params = self.params.to_tir(ctx);
        tir::Body { params, expr: box self.expr.to_tir(ctx) }
    }
}

impl<'tcx> Tir<'tcx> for ir::Generics<'tcx> {
    type Output = tir::Generics<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        tir::Generics { data: 0, pd: PhantomData }
    }
}

impl<'tcx> Tir<'tcx> for ir::Let<'tcx> {
    type Output = tir::Let<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let tcx = ctx.tcx;
        tir::Let {
            id: self.id,
            pat: box self.pat.to_tir(ctx),
            init: self.init.map(|init| box init.to_tir(ctx)),
        }
    }
}

impl<'tcx> Tir<'tcx> for ir::Stmt<'tcx> {
    type Output = tir::Stmt<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let &Self { id, span, ref kind } = self;
        let kind = match kind {
            ir::StmtKind::Let(l) => tir::StmtKind::Let(l.to_tir(ctx)),
            // we can map both semi and expr to expressions and their distinction is no longer
            // important after typechecking is done
            ir::StmtKind::Expr(expr) | ir::StmtKind::Semi(expr) =>
                tir::StmtKind::Expr(box expr.to_tir(ctx)),
        };
        tir::Stmt { id, span, kind }
    }
}

impl<'tcx> Tir<'tcx> for ir::Block<'tcx> {
    type Output = tir::Block<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let tcx = ctx.tcx;
        let stmts = self.stmts.iter().map(|stmt| stmt.to_tir(ctx)).collect();
        let expr = self.expr.map(|expr| box expr.to_tir(ctx));
        tir::Block { id: self.id, stmts, expr }
    }
}

impl<'tcx, T> Tir<'tcx> for &[T]
where
    T: Tir<'tcx>,
{
    type Output = Vec<T::Output>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        self.iter().map(|x| x.to_tir(ctx)).collect()
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

impl<'tcx> TirCtx<'_, 'tcx> {
    fn lower_path(&self, expr: &ir::Expr<'tcx>, path: &ir::Path<'tcx>) -> tir::ExprKind<'tcx> {
        match path.res {
            Res::Local(id) => tir::ExprKind::VarRef(id),
            Res::Def(def_id, def_kind) => match def_kind {
                DefKind::Ctor(CtorKind::Unit, ..) => {
                    let (adt, substs) = self.node_type(expr.id).expect_adt();
                    tir::ExprKind::Adt {
                        adt,
                        substs,
                        fields: vec![],
                        variant_idx: adt.variant_idx_with_ctor(def_id),
                    }
                }
                // functions and function-like variant constructors
                DefKind::Ctor(CtorKind::Tuple, ..) | ir::DefKind::Fn =>
                    tir::ExprKind::ItemRef(def_id),
                DefKind::AssocFn => todo!(),
                DefKind::Extern => todo!(),
                DefKind::Impl => todo!(),
                DefKind::Ctor(..) => todo!(),
                DefKind::TyParam(_) => todo!(),
                DefKind::Enum => todo!(),
                DefKind::Struct => todo!(),
            },
            Res::SelfTy => todo!(),
            Res::Err | Res::PrimTy(_) => unreachable!(),
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
                tir::ExprKind::Bin(*op, box l.to_tir(self), box r.to_tir(self)),
            ir::ExprKind::Unary(UnaryOp::Deref, expr) =>
                tir::ExprKind::Deref(box expr.to_tir(self)),
            ir::ExprKind::Unary(UnaryOp::Ref, expr) => tir::ExprKind::Ref(box expr.to_tir(self)),
            ir::ExprKind::Unary(op, expr) => tir::ExprKind::Unary(*op, box expr.to_tir(self)),
            ir::ExprKind::Block(block) => tir::ExprKind::Block(box block.to_tir(self)),
            ir::ExprKind::Path(path) => self.lower_path(expr, path),
            ir::ExprKind::Tuple(xs) => tir::ExprKind::Tuple(xs.to_tir(self)),
            ir::ExprKind::Closure(_sig, body) => self.lower_closure(expr, body),
            ir::ExprKind::Call(f, args) =>
                tir::ExprKind::Call(box f.to_tir(self), args.to_tir(self)),
            ir::ExprKind::Lit(lit) => tir::ExprKind::Const(lit.to_tir(self)),
            ir::ExprKind::Match(expr, arms, _) =>
                tir::ExprKind::Match(box expr.to_tir(self), arms.to_tir(self)),
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
            ir::ExprKind::Ret(expr) => tir::ExprKind::Ret(expr.map(|expr| box expr.to_tir(self))),
            ir::ExprKind::Assign(l, r) =>
                tir::ExprKind::Assign(box l.to_tir(self), box r.to_tir(self)),
            ir::ExprKind::Field(base, _) =>
                tir::ExprKind::Field(box base.to_tir(self), self.tables.field_index(expr.id)),
            ir::ExprKind::Box(expr) => tir::ExprKind::Box(box expr.to_tir(self)),
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
            AdjustmentKind::Deref => tir::ExprKind::Deref(box expr),
            AdjustmentKind::NeverToAny => todo!(),
        };
        tir::Expr { ty: adjustment.ty, span, kind }
    }
}

impl<'tcx> Tir<'tcx> for Lit {
    type Output = &'tcx Const<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let c = match *self {
            Lit::Float(n) => Const::new(ConstKind::Float(n), ctx.tcx.types.float),
            Lit::Bool(b) => Const::new(ConstKind::Bool(b), ctx.tcx.types.boolean),
            Lit::Int(i) => Const::new(ConstKind::Int(i), ctx.tcx.types.int),
        };
        ctx.intern_const(c)
    }
}

impl<'tcx> Tir<'tcx> for ir::Arm<'tcx> {
    type Output = tir::Arm<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let &ir::Arm { id, span, ref pat, ref body, ref guard } = self;
        tir::Arm {
            id,
            span,
            pat: box pat.to_tir(ctx),
            body: box body.to_tir(ctx),
            guard: guard.map(|expr| box expr.to_tir(ctx)),
        }
    }
}