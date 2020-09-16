use crate::ast::{Lit, UnaryOp, Visibility};
use crate::ir::{self, CtorKind, DefId, VariantIdx};
use crate::mir;
use crate::tir;
use crate::ty::*;
use crate::typeck::inference::InferCtx;
use crate::typeck::{TyCtx, TypeckOutputs};
use indexed_vec::Idx;
use ir::DefKind;
use smallvec::SmallVec;
use std::marker::PhantomData;
use std::ops::Deref;

/// ir -> tir -> mir
pub struct TirCtx<'a, 'tcx> {
    tcx: TyCtx<'tcx>,
    infcx: &'a InferCtx<'a, 'tcx>,
    tables: &'a TypeckOutputs<'tcx>,
}

impl<'a, 'tcx> Deref for TirCtx<'a, 'tcx> {
    type Target = &'a InferCtx<'a, 'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.infcx
    }
}

impl<'a, 'tcx> TirCtx<'a, 'tcx> {
    pub fn new(infcx: &'a InferCtx<'a, 'tcx>, tables: &'a TypeckOutputs<'tcx>) -> Self {
        Self { infcx, tcx: infcx.tcx, tables }
    }

    /// ir -> mir
    pub fn lower_item_tir(&mut self, item: &ir::Item<'tcx>) -> SmallVec<[tir::Item<'tcx>; 1]> {
        // this `tir` may still have unsubstituted inference variables in it
        item.to_tir(self)
    }

    /// ir -> tir -> mir
    pub fn lower_item(&mut self, item: &ir::Item<'tcx>) -> SmallVec<[mir::Item<'tcx>; 1]> {
        let &ir::Item { id, span, vis, ident, ref kind } = item;
        let tir = self.lower_item_tir(item);
        tir.iter()
            .map(|item| {
                println!("{}", item);
                match item.kind {
                    tir::ItemKind::Fn(_, _, body) => {
                        let mir_body = mir::build_fn(self, body);
                        mir::Item { span, id, vis, ident, kind: mir::ItemKind::Fn(mir_body) }
                    }
                }
            })
            .collect()
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
    type Output = SmallVec<[tir::Item<'tcx>; 1]>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let &Self { span, id, ident, vis, ref kind } = self;
        match kind {
            ir::ItemKind::Fn(sig, generics, body) => {
                let ty = ctx.tcx.collected_ty(self.id.def);
                let kind = tir::ItemKind::Fn(ty, generics.to_tir(ctx), body.to_tir(ctx));
                smallvec![tir::Item { kind, span, id, ident, vis }]
            }
            ir::ItemKind::Struct(..) => unreachable!(),
            ir::ItemKind::Enum(_, variants) =>
                variants.iter().map(|v| ctx.mk_enum_variant_ctor(self, v)).collect(),
        }
    }
}

impl<'tcx> TirCtx<'_, 'tcx> {
    /// manually constructs the tir for an enum constructor function
    fn mk_enum_variant_ctor(
        &mut self,
        item: &ir::Item<'tcx>,
        variant: &ir::Variant<'tcx>,
    ) -> tir::Item<'tcx> {
        let ctor_ty = self.ctor_ty(variant.id.def);
        let body = todo!();
        let kind = tir::ItemKind::Fn(ctor_ty, item.generics().to_tir(self), body);
        tir::Item {
            span: variant.span,
            id: variant.id,
            vis: item.vis,
            ident: item.ident.concat_as_path(variant.ident),
            kind,
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
        };
        let ty = ctx.node_type(id);
        tir::Pattern { id, span, kind, ty }
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
                ir::DefKind::Fn | ir::DefKind::Ctor(CtorKind::Fn | CtorKind::Struct, ..) =>
                    tir::ExprKind::ItemRef(def_id),
                ir::DefKind::TyParam(_) => todo!(),
                ir::DefKind::Enum => todo!(),
                ir::DefKind::Struct => todo!(),
            },
            ir::Res::Err | ir::Res::PrimTy(_) => unreachable!(),
        }
    }
}

impl<'tcx> Tir<'tcx> for ir::Expr<'tcx> {
    type Output = tir::Expr<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        let &Self { span, id, ref kind } = self;
        let ty = ctx.node_type(self.id);
        let kind = match kind {
            ir::ExprKind::Bin(op, l, r) =>
                tir::ExprKind::Bin(*op, l.to_tir_alloc(ctx), r.to_tir_alloc(ctx)),
            ir::ExprKind::Unary(UnaryOp::Deref, expr) =>
                tir::ExprKind::Deref(expr.to_tir_alloc(ctx)),
            ir::ExprKind::Unary(UnaryOp::Ref, expr) => tir::ExprKind::Ref(expr.to_tir_alloc(ctx)),
            ir::ExprKind::Unary(op, expr) => tir::ExprKind::Unary(*op, expr.to_tir_alloc(ctx)),
            ir::ExprKind::Block(block) => tir::ExprKind::Block(block.to_tir_alloc(ctx)),
            ir::ExprKind::Path(path) => ctx.lower_path(self, path),
            ir::ExprKind::Tuple(xs) => tir::ExprKind::Tuple(xs.to_tir(ctx)),
            ir::ExprKind::Closure(_, body) => tir::ExprKind::Lambda(body.to_tir_alloc(ctx)),
            ir::ExprKind::Call(f, args) =>
                tir::ExprKind::Call(f.to_tir_alloc(ctx), args.to_tir(ctx)),
            ir::ExprKind::Lit(lit) => tir::ExprKind::Const(lit.to_tir_alloc(ctx)),
            ir::ExprKind::Match(expr, arms, _) =>
                tir::ExprKind::Match(expr.to_tir_alloc(ctx), arms.to_tir(ctx)),
            ir::ExprKind::Struct(path, fields) => match ty.kind {
                TyKind::Adt(adt, substs) => match adt.kind {
                    AdtKind::Struct => tir::ExprKind::Adt {
                        adt,
                        substs,
                        variant_idx: VariantIdx::new(0),
                        fields: fields.to_tir(ctx),
                    },
                    AdtKind::Enum => {
                        let variant_idx = match path.res {
                            ir::Res::Def(_, DefKind::Ctor(CtorKind::Struct, idx, _)) => idx,
                            _ => panic!("unexpected resolution"),
                        };
                        tir::ExprKind::Adt { adt, substs, variant_idx, fields: fields.to_tir(ctx) }
                    }
                },
                _ => unreachable!(),
            },
            ir::ExprKind::Ret(expr) => tir::ExprKind::Ret(expr.map(|expr| expr.to_tir_alloc(ctx))),
            ir::ExprKind::Assign(l, r) =>
                tir::ExprKind::Assign(l.to_tir_alloc(ctx), r.to_tir_alloc(ctx)),
            ir::ExprKind::Field(expr, _) =>
                tir::ExprKind::Field(expr.to_tir_alloc(ctx), ctx.tables.field_index(self.id)),
            ir::ExprKind::Box(expr) => tir::ExprKind::Box(expr.to_tir_alloc(ctx)),
        };
        tir::Expr { span, id, kind, ty }
    }
}

impl<'tcx> Tir<'tcx> for Lit {
    type Output = Const<'tcx>;

    fn to_tir(&self, ctx: &mut TirCtx<'_, 'tcx>) -> Self::Output {
        match *self {
            Lit::Float(n) => Const::new(ConstKind::Float(n)),
            Lit::Bool(b) => Const::new(ConstKind::Bool(b)),
            Lit::Int(i) => Const::new(ConstKind::Int(i)),
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
