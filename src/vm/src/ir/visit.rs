use crate::ast::{Ident, Visibility};
use crate::ir;

pub trait Visitor<'ir>: Sized {
    fn visit_item(&mut self, item: &'ir ir::Item<'ir>) {
        walk_item(self, item)
    }

    fn visit_body(&mut self, item: &'ir ir::Body<'ir>) {
        walk_body(self, item)
    }

    fn visit_param(&mut self, param: &'ir ir::Param<'ir>) {
        walk_param(self, param)
    }

    fn visit_expr(&mut self, expr: &'ir ir::Expr<'ir>) {
        walk_expr(self, expr)
    }

    fn visit_vis(&mut self, vis: Visibility) {
    }

    fn visit_lambda(&mut self, sig: &'ir ir::FnSig<'ir>, body: &'ir ir::Body) {
        walk_lambda(self, sig, body)
    }

    fn visit_ident(&mut self, ident: Ident) {
    }

    fn visit_let(&mut self, l: &'ir ir::Let<'ir>) {
        walk_let(self, l)
    }

    fn visit_pat(&mut self, pat: &'ir ir::Pattern<'ir>) {
        walk_pat(self, pat)
    }

    fn visit_block(&mut self, block: &'ir ir::Block<'ir>) {
        walk_block(self, block)
    }

    fn visit_stmt(&mut self, stmt: &'ir ir::Stmt<'ir>) {
        walk_stmt(self, stmt)
    }

    fn visit_ty(&mut self, ty: &'ir ir::Ty<'ir>) {
        walk_ty(self, ty)
    }

    fn visit_path(&mut self, path: &'ir ir::Path<'ir>) {
        walk_path(self, path)
    }

    fn visit_path_segment(&mut self, seg: &'ir ir::PathSegment<'ir>) {
        walk_path_segment(self, seg)
    }

    fn visit_fn_sig(&mut self, sig: &'ir ir::FnSig<'ir>) {
        walk_fn_sig(self, sig)
    }

    fn visit_arm(&mut self, arm: &'ir ir::Arm<'ir>) {
        walk_arm(self, arm)
    }

    fn visit_field(&mut self, field: &'ir ir::Field<'ir>) {
        walk_field(self, field)
    }
}

pub fn walk_prog<'ir, V: Visitor<'ir>>(v: &mut V, prog: &'ir ir::Prog<'ir>) {
    prog.items.values().for_each(|item| v.visit_item(item))
}

pub fn walk_fn_sig<'ir, V: Visitor<'ir>>(v: &mut V, sig: &'ir ir::FnSig<'ir>) {
    sig.inputs.iter().for_each(|ty| v.visit_ty(ty));
    sig.output.map(|ty| v.visit_ty(ty));
}

pub fn walk_item<'ir, V: Visitor<'ir>>(v: &mut V, item: &'ir ir::Item<'ir>) {
    v.visit_vis(item.vis);
    v.visit_ident(item.ident);
    match &item.kind {
        ir::ItemKind::Fn(sig, generics, body) => {
            v.visit_fn_sig(sig);
            v.visit_body(body);
        }
        ir::ItemKind::Struct(_, _) => {}
    }
}

pub fn walk_body<'ir, V: Visitor<'ir>>(v: &mut V, body: &'ir ir::Body<'ir>) {
    body.params.iter().for_each(|p| v.visit_param(p));
    v.visit_expr(body.expr)
}

pub fn walk_param<'ir, V: Visitor<'ir>>(v: &mut V, param: &'ir ir::Param<'ir>) {
    v.visit_pat(param.pat)
}

pub fn walk_lambda<'ir, V: Visitor<'ir>>(
    v: &mut V,
    sig: &'ir ir::FnSig<'ir>,
    body: &'ir ir::Body<'ir>,
) {
    v.visit_fn_sig(sig);
    v.visit_body(body);
}

pub fn walk_expr<'ir, V: Visitor<'ir>>(v: &mut V, expr: &'ir ir::Expr<'ir>) {
    match &expr.kind {
        ir::ExprKind::Bin(_, l, r) => {
            v.visit_expr(l);
            v.visit_expr(r);
        }
        ir::ExprKind::Unary(_, expr) => v.visit_expr(expr),
        ir::ExprKind::Block(block) => v.visit_block(block),
        ir::ExprKind::Path(path) => v.visit_path(path),
        ir::ExprKind::Tuple(xs) => xs.iter().for_each(|x| v.visit_expr(x)),
        ir::ExprKind::Closure(sig, body) => v.visit_lambda(sig, body),
        ir::ExprKind::Call(f, args) => {
            v.visit_expr(f);
            args.iter().for_each(|arg| v.visit_expr(arg));
        }
        ir::ExprKind::Lit(_) => {}
        ir::ExprKind::Ret(expr) => expr.iter().for_each(|expr| v.visit_expr(expr)),
        ir::ExprKind::Match(expr, arms, _) => {
            v.visit_expr(expr);
            arms.iter().for_each(|arm| v.visit_arm(arm));
        }
        ir::ExprKind::Struct(path, fields) => {
            v.visit_path(path);
            fields.iter().for_each(|f| v.visit_field(f));
        }
        ir::ExprKind::Assign(l, r) => {
            v.visit_expr(l);
            v.visit_expr(r);
        }
        ir::ExprKind::Field(expr, ident) => {
            v.visit_expr(expr);
            v.visit_ident(*ident);
        }
    }
}

pub fn walk_field<'ir, V: Visitor<'ir>>(v: &mut V, field: &'ir ir::Field<'ir>) {
    v.visit_ident(field.ident);
    v.visit_expr(field.expr);
}

pub fn walk_arm<'ir, V: Visitor<'ir>>(v: &mut V, arm: &'ir ir::Arm<'ir>) {
    v.visit_pat(arm.pat);
    arm.guard.map(|expr| v.visit_expr(expr));
    v.visit_expr(arm.body);
}

pub fn walk_stmt<'ir, V: Visitor<'ir>>(v: &mut V, stmt: &'ir ir::Stmt<'ir>) {
    match &stmt.kind {
        ir::StmtKind::Let(l) => v.visit_let(l),
        ir::StmtKind::Expr(expr) | ir::StmtKind::Semi(expr) => v.visit_expr(expr),
    }
}

pub fn walk_block<'ir, V: Visitor<'ir>>(v: &mut V, block: &'ir ir::Block<'ir>) {
    block.stmts.iter().for_each(|stmt| v.visit_stmt(stmt));
    block.expr.map(|expr| v.visit_expr(expr));
}

pub fn walk_ty<'ir, V: Visitor<'ir>>(v: &mut V, ty: &'ir ir::Ty<'ir>) {
    match &ty.kind {
        ir::TyKind::Path(path) => v.visit_path(path),
        ir::TyKind::Array(ty) => v.visit_ty(ty),
        ir::TyKind::Tuple(tys) => tys.iter().for_each(|ty| v.visit_ty(ty)),
        ir::TyKind::Fn(params, ret) => {
            params.iter().for_each(|ty| v.visit_ty(ty));
            v.visit_ty(ty);
        }
        ir::TyKind::Infer => {}
    }
}

pub fn walk_let<'ir, V: Visitor<'ir>>(v: &mut V, l: &'ir ir::Let<'ir>) {
    l.init.map(|expr| v.visit_expr(expr));
    v.visit_pat(l.pat);
    l.ty.map(|ty| v.visit_ty(ty));
}

pub fn walk_path<'ir, V: Visitor<'ir>>(v: &mut V, path: &'ir ir::Path<'ir>) {
    path.segments.iter().for_each(|seg| v.visit_path_segment(seg));
}

pub fn walk_path_segment<'ir, V: Visitor<'ir>>(v: &mut V, seg: &'ir ir::PathSegment<'ir>) {
    v.visit_ident(seg.ident)
}

pub fn walk_pat<'ir, V: Visitor<'ir>>(v: &mut V, pat: &'ir ir::Pattern<'ir>) {
    match &pat.kind {
        ir::PatternKind::Wildcard => {}
        ir::PatternKind::Tuple(pats) => pats.iter().for_each(|p| v.visit_pat(p)),
        ir::PatternKind::Lit(expr) => v.visit_expr(expr),
        ir::PatternKind::Binding(ident, subpat, m) => {
            v.visit_ident(*ident);
            subpat.map(|p| v.visit_pat(p));
        }
    }
}
