use crate::ir::{self, DefId};
use crate::tir;
use crate::ty::{InferenceVarSubstFolder, Subst};
use crate::typeck::{inference::InferCtx, TyCtx, TypeckTables};
use std::marker::PhantomData;

/// ir -> tir
/// simple procedure of basically just injecting type annotations into expressions and patterns
crate struct IrLoweringCtx<'a, 'tcx> {
    tcx: TyCtx<'tcx>,
    infcx: &'a InferCtx<'a, 'tcx>,
    tables: &'a TypeckTables<'tcx>,
}

impl<'a, 'tcx> IrLoweringCtx<'a, 'tcx> {
    pub fn new(infcx: &'a InferCtx<'a, 'tcx>, tables: &'a TypeckTables<'tcx>) -> Self {
        Self { infcx, tcx: infcx.tcx, tables }
    }

    pub fn lower_item(&mut self, item: &ir::Item<'tcx>) -> &'tcx tir::Item<'tcx> {
        // this may still have unsubstituted inference variables in it
        item.to_tir(self)
    }
}

/// trait for conversion to tir
crate trait Tir<'tcx> {
    type Output;
    fn to_tir(&self, ctx: &mut IrLoweringCtx<'_, 'tcx>) -> Self::Output;
}

impl<'tcx> Tir<'tcx> for ir::Item<'tcx> {
    type Output = &'tcx tir::Item<'tcx>;
    fn to_tir(&self, ctx: &mut IrLoweringCtx<'_, 'tcx>) -> Self::Output {
        let &Self { span, id, ident, vis, ref kind } = self;
        let kind = match kind {
            ir::ItemKind::Fn(sig, generics, body) => {
                let fn_ty = ctx.tcx.item_ty(self.id.def_id);
                tir::ItemKind::Fn(fn_ty, generics.to_tir(ctx), body.to_tir(ctx))
            }
        };
        ctx.tcx.alloc_tir(tir::Item { kind, span, id, ident, vis })
    }
}

impl<'tcx> Tir<'tcx> for ir::Param<'tcx> {
    type Output = tir::Param<'tcx>;
    fn to_tir(&self, ctx: &mut IrLoweringCtx<'_, 'tcx>) -> Self::Output {
        let &Self { id, span, ref pat } = self;
        tir::Param { id, span, pat: pat.to_tir(ctx) }
    }
}

impl<'tcx> Tir<'tcx> for ir::Pattern<'tcx> {
    type Output = &'tcx tir::Pattern<'tcx>;
    fn to_tir(&self, ctx: &mut IrLoweringCtx<'_, 'tcx>) -> Self::Output {
        let &Self { id, span, ref kind } = self;
        let kind = match kind {
            ir::PatternKind::Wildcard => tir::PatternKind::Wildcard,
            ir::PatternKind::Binding(ident, sub) => {
                let subpat = sub.map(|pat| pat.to_tir(ctx));
                tir::PatternKind::Binding(*ident, subpat)
            }
        };
        let ty = ctx.tables.node_type(id);
        ctx.tcx.alloc_tir(tir::Pattern { id, span, kind, ty })
    }
}

impl<'tcx> Tir<'tcx> for ir::Body<'tcx> {
    type Output = &'tcx tir::Body<'tcx>;
    fn to_tir(&self, ctx: &mut IrLoweringCtx<'_, 'tcx>) -> Self::Output {
        let tcx = ctx.tcx;
        let params = tcx.alloc_tir_iter(self.params.into_iter().map(|p| p.to_tir(ctx)));
        let body = tir::Body { params, expr: self.expr.to_tir(ctx) };
        ctx.tcx.alloc_tir(body)
    }
}

impl<'tcx> Tir<'tcx> for ir::Generics<'tcx> {
    type Output = &'tcx tir::Generics<'tcx>;
    fn to_tir(&self, ctx: &mut IrLoweringCtx<'_, 'tcx>) -> Self::Output {
        ctx.tcx.alloc_tir(tir::Generics { data: 0, pd: PhantomData })
    }
}

impl<'tcx> Tir<'tcx> for ir::Let<'tcx> {
    type Output = &'tcx tir::Let<'tcx>;
    fn to_tir(&self, ctx: &mut IrLoweringCtx<'_, 'tcx>) -> Self::Output {
        let tcx = ctx.tcx;
        tcx.alloc_tir(tir::Let {
            id: self.id,
            pat: self.pat.to_tir(ctx),
            init: self.init.map(|init| init.to_tir(ctx)),
        })
    }
}

impl<'tcx> Tir<'tcx> for ir::Stmt<'tcx> {
    type Output = tir::Stmt<'tcx>;
    fn to_tir(&self, ctx: &mut IrLoweringCtx<'_, 'tcx>) -> Self::Output {
        let &Self { id, span, ref kind } = self;
        let kind = match kind {
            ir::StmtKind::Let(l) => tir::StmtKind::Let(l.to_tir(ctx)),
            // we can map both semi and expr to expressions and their distinction is no longer
            // important after typechecking is done
            ir::StmtKind::Expr(expr) => tir::StmtKind::Expr(expr.to_tir(ctx)),
            ir::StmtKind::Semi(expr) => tir::StmtKind::Expr(expr.to_tir(ctx)),
        };
        tir::Stmt { id, span, kind }
    }
}

impl<'tcx> Tir<'tcx> for ir::Block<'tcx> {
    type Output = &'tcx tir::Block<'tcx>;
    fn to_tir(&self, ctx: &mut IrLoweringCtx<'_, 'tcx>) -> Self::Output {
        let tcx = ctx.tcx;
        let stmts = tcx.alloc_tir_iter(self.stmts.iter().map(|stmt| stmt.to_tir(ctx)));
        let expr = self.expr.map(|expr| expr.to_tir(ctx));
        let block = tir::Block { id: self.id, stmts, expr };
        ctx.tcx.alloc_tir(block)
    }
}

impl<'tcx> Tir<'tcx> for ir::Expr<'tcx> {
    type Output = &'tcx tir::Expr<'tcx>;
    fn to_tir(&self, ctx: &mut IrLoweringCtx<'_, 'tcx>) -> Self::Output {
        let kind = match &self.kind {
            &ir::ExprKind::Lit(lit) => tir::ExprKind::Lit(lit),
            ir::ExprKind::Bin(op, l, r) => tir::ExprKind::Bin(*op, l.to_tir(ctx), r.to_tir(ctx)),
            ir::ExprKind::Unary(op, expr) => tir::ExprKind::Unary(*op, expr.to_tir(ctx)),
            ir::ExprKind::Block(block) => tir::ExprKind::Block(block.to_tir(ctx)),
        };
        let ty = ctx.tables.node_type(self.id);
        ctx.tcx.alloc_tir(tir::Expr { span: self.span, kind, ty })
    }
}
