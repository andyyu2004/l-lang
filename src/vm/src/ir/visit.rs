use crate::ast::{Ident, Visibility};
use crate::ir;

crate trait Visitor<'ir>: Sized {
    fn visit_item(&mut self, item: &'ir ir::Item<'ir>) { walk_item(self, item) }

    fn visit_body(&mut self, item: &'ir ir::Body<'ir>) { walk_body(self, item) }

    fn visit_param(&mut self, param: &'ir ir::Param<'ir>) { walk_param(self, param) }

    fn visit_expr(&mut self, expr: &'ir ir::Expr<'ir>) { walk_expr(self, expr) }

    fn visit_vis(&mut self, vis: Visibility) {
    }

    fn visit_ident(&mut self, ident: Ident) {
    }

    fn visit_let(&mut self, l: &'ir ir::Let<'ir>) { walk_let(self, l) }

    fn visit_pat(&mut self, pat: &'ir ir::Pattern<'ir>) { walk_pat(self, pat) }

    fn visit_block(&mut self, block: &'ir ir::Block<'ir>) { walk_block(self, block) }

    fn visit_stmt(&mut self, stmt: &'ir ir::Stmt<'ir>) { walk_stmt(self, stmt) }

    fn visit_ty(&mut self, ty: &'ir ir::Ty<'ir>) { walk_ty(self, ty) }

    fn visit_path(&mut self, path: &'ir ir::Path<'ir>) { walk_path(self, path) }

    fn visit_path_segment(&mut self, seg: &'ir ir::PathSegment<'ir>) {
        walk_path_segment(self, seg)
    }
}

crate fn walk_prog<'ir, V: Visitor<'ir>>(v: &mut V, prog: &'ir ir::Prog<'ir>) {
    prog.items.values().for_each(|item| v.visit_item(item))
}

crate fn walk_item<'ir, V: Visitor<'ir>>(v: &mut V, item: &'ir ir::Item<'ir>) {
    v.visit_vis(item.vis);
    v.visit_ident(item.ident);
    match &item.kind {
        ir::ItemKind::Fn(sig, generics, body) => {
            sig.inputs.iter().for_each(|ty| v.visit_ty(ty));
            sig.output.map(|ty| v.visit_ty(ty));
            v.visit_body(body);
        }
    }
}

crate fn walk_body<'ir, V: Visitor<'ir>>(v: &mut V, body: &'ir ir::Body<'ir>) {
    body.params.iter().map(|p| v.visit_param(p));
    v.visit_expr(body.expr)
}

crate fn walk_param<'ir, V: Visitor<'ir>>(v: &mut V, param: &'ir ir::Param<'ir>) {
    v.visit_pat(param.pat)
}

crate fn walk_expr<'ir, V: Visitor<'ir>>(v: &mut V, expr: &'ir ir::Expr<'ir>) {
    match &expr.kind {
        ir::ExprKind::Bin(_, l, r) => {
            v.visit_expr(l);
            v.visit_expr(r);
        }
        ir::ExprKind::Unary(_, expr) => v.visit_expr(expr),
        ir::ExprKind::Block(block) => v.visit_block(block),
        ir::ExprKind::Path(path) => v.visit_path(path),
        ir::ExprKind::Tuple(xs) => xs.iter().for_each(|x| v.visit_expr(x)),
        ir::ExprKind::Lit(_) => {}
    }
}

crate fn walk_stmt<'ir, V: Visitor<'ir>>(v: &mut V, stmt: &'ir ir::Stmt<'ir>) {
    match &stmt.kind {
        ir::StmtKind::Let(l) => v.visit_let(l),
        ir::StmtKind::Expr(expr) | ir::StmtKind::Semi(expr) => v.visit_expr(expr),
    }
}

crate fn walk_block<'ir, V: Visitor<'ir>>(v: &mut V, block: &'ir ir::Block<'ir>) {
    block.stmts.iter().for_each(|stmt| v.visit_stmt(stmt));
    block.expr.map(|expr| v.visit_expr(expr));
}

crate fn walk_ty<'ir, V: Visitor<'ir>>(v: &mut V, ty: &'ir ir::Ty<'ir>) {
    match &ty.kind {
        ir::TyKind::Path(path) => v.visit_path(path),
        ir::TyKind::Array(ty) => v.visit_ty(ty),
    }
}

crate fn walk_let<'ir, V: Visitor<'ir>>(v: &mut V, l: &'ir ir::Let<'ir>) {
    v.visit_pat(l.pat);
    l.ty.map(|ty| v.visit_ty(ty));
    l.init.map(|expr| v.visit_expr(expr));
}

crate fn walk_path<'ir, V: Visitor<'ir>>(v: &mut V, path: &'ir ir::Path<'ir>) {
    path.segments.iter().map(|seg| v.visit_path_segment(seg));
}

crate fn walk_path_segment<'ir, V: Visitor<'ir>>(v: &mut V, seg: &'ir ir::PathSegment<'ir>) {
    v.visit_ident(seg.ident)
}

crate fn walk_pat<'ir, V: Visitor<'ir>>(v: &mut V, pat: &'ir ir::Pattern<'ir>) {
    match &pat.kind {
        ir::PatternKind::Wildcard => {}
        &ir::PatternKind::Binding(ident, ref subpat) => {
            v.visit_ident(ident);
            subpat.map(|p| v.visit_pat(p));
        }
        ir::PatternKind::Tuple(pats) => pats.iter().for_each(|p| v.visit_pat(p)),
    }
}
