use crate::*;
use ast::{Ident, Visibility};

pub trait Visitor<'ir>: Sized {
    fn visit_ir(&mut self, prog: &'ir IR<'ir>) {
        walk_ir(self, prog)
    }

    fn visit_item(&mut self, item: &'ir ir::Item<'ir>) {
        walk_item(self, item)
    }

    fn visit_body(&mut self, body: &'ir ir::Body<'ir>) {
        walk_body(self, body)
    }

    fn visit_foreign_item(&mut self, item: &'ir ir::ForeignItem<'ir>) {
        walk_foreign_item(self, item);
    }

    fn visit_param(&mut self, param: &'ir ir::Param<'ir>) {
        walk_param(self, param)
    }

    fn visit_expr(&mut self, expr: &'ir ir::Expr<'ir>) {
        walk_expr(self, expr)
    }

    fn visit_ty_param(&mut self, param: &'ir ir::TyParam<'ir>) {
        walk_ty_param(self, param)
    }

    fn visit_vis(&mut self, _vis: Visibility) {
    }

    fn visit_lambda(&mut self, sig: &'ir ir::FnSig<'ir>, body: &'ir ir::Body) {
        walk_lambda(self, sig, body)
    }

    fn visit_ident(&mut self, _ident: Ident) {
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

    fn visit_generics(&mut self, generics: &'ir ir::Generics<'ir>) {
        walk_generics(self, generics)
    }

    fn visit_variant_kind(&mut self, kind: &'ir ir::VariantKind<'ir>) {
        walk_variant_kind(self, kind)
    }

    fn visit_field_decl(&mut self, decl: &'ir ir::FieldDecl<'ir>) {
        walk_field_decl(self, decl)
    }

    fn visit_variant(&mut self, variant: &'ir ir::Variant<'ir>) {
        walk_variant(self, variant)
    }

    fn visit_impl_item(&mut self, impl_item: &'ir ir::ImplItem<'ir>) {
        walk_impl_item(self, impl_item);
    }
}

pub fn walk_ir<'ir>(v: &mut impl Visitor<'ir>, prog: &'ir ir::IR<'ir>) {
    prog.items.values().for_each(|item| v.visit_item(item));
    prog.impl_items.values().for_each(|impl_item| v.visit_impl_item(impl_item));
}

pub fn walk_impl_item<'ir>(v: &mut impl Visitor<'ir>, impl_item: &'ir ir::ImplItem<'ir>) {
    match impl_item.kind {
        ir::ImplItemKind::Fn(sig, body) => {
            v.visit_fn_sig(sig);
            v.visit_body(body);
        }
    }
}

pub fn walk_foreign_item<'ir>(v: &mut impl Visitor<'ir>, item: &'ir ForeignItem<'ir>) {
    match item.kind {
        ForeignItemKind::Fn(sig, generics) => {
            v.visit_fn_sig(sig);
            v.visit_generics(generics);
        }
    }
}

pub fn walk_variant<'ir>(v: &mut impl Visitor<'ir>, variant: &'ir ir::Variant<'ir>) {
    v.visit_ident(variant.ident);
    v.visit_variant_kind(&variant.kind);
}

pub fn walk_field_decl<'ir>(v: &mut impl Visitor<'ir>, decl: &'ir ir::FieldDecl<'ir>) {
    v.visit_ident(decl.ident);
    v.visit_vis(decl.vis);
    v.visit_ty(decl.ty);
}

pub fn walk_variant_kind<'ir>(v: &mut impl Visitor<'ir>, kind: &'ir ir::VariantKind<'ir>) {
    match kind {
        ir::VariantKind::Struct(fields) | ir::VariantKind::Tuple(fields) =>
            fields.iter().for_each(|f| v.visit_field_decl(f)),
        ir::VariantKind::Unit => {}
    }
}

pub fn walk_ty_param<'ir>(v: &mut impl Visitor<'ir>, param: &'ir ir::TyParam<'ir>) {
    param.default.iter().for_each(|ty| v.visit_ty(ty));
}

pub fn walk_generics<'ir>(v: &mut impl Visitor<'ir>, generics: &'ir ir::Generics<'ir>) {
    generics.params.iter().for_each(|param| v.visit_ty_param(param));
}

pub fn walk_fn_sig<'ir, V: Visitor<'ir>>(v: &mut V, sig: &'ir ir::FnSig<'ir>) {
    sig.inputs.iter().for_each(|ty| v.visit_ty(ty));
    sig.output.iter().for_each(|ty| v.visit_ty(ty));
}

pub fn walk_item<'ir, V: Visitor<'ir>>(v: &mut V, item: &'ir ir::Item<'ir>) {
    v.visit_vis(item.vis);
    v.visit_ident(item.ident);
    match &item.kind {
        ir::ItemKind::Fn(sig, generics, body) => {
            v.visit_fn_sig(sig);
            v.visit_generics(generics);
            v.visit_body(body);
        }
        ir::ItemKind::Enum(generics, variants) => {
            v.visit_generics(generics);
            variants.iter().for_each(|variant| v.visit_variant(variant));
        }
        ir::ItemKind::Struct(generics, kind) => {
            v.visit_generics(generics);
            v.visit_variant_kind(kind);
        }
        ir::ItemKind::Extern(items) => items.iter().for_each(|item| v.visit_foreign_item(item)),
        ir::ItemKind::Impl { generics, trait_path, self_ty, impl_item_refs } => {
            v.visit_generics(generics);
            trait_path.iter().for_each(|path| v.visit_path(path));
            v.visit_ty(self_ty);
            // TODO
            // impl_item_refs.iter().for_each();
        }
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
        ir::ExprKind::Unary(_, expr) => v.visit_expr(expr),
        ir::ExprKind::Block(block) => v.visit_block(block),
        ir::ExprKind::Path(path) => v.visit_path(path),
        ir::ExprKind::Tuple(xs) => xs.iter().for_each(|x| v.visit_expr(x)),
        ir::ExprKind::Closure(sig, body) => v.visit_lambda(sig, body),
        ir::ExprKind::Box(expr) => v.visit_expr(expr),
        ir::ExprKind::Call(f, args) => {
            v.visit_expr(f);
            args.iter().for_each(|arg| v.visit_expr(arg));
        }
        ir::ExprKind::Bin(_, l, r) => {
            v.visit_expr(l);
            v.visit_expr(r);
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
        ir::ExprKind::Field(base, ident) => {
            v.visit_expr(base);
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
    arm.guard.iter().for_each(|expr| v.visit_expr(expr));
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
    block.expr.iter().for_each(|expr| v.visit_expr(expr));
}

pub fn walk_ty<'ir, V: Visitor<'ir>>(v: &mut V, ty: &'ir ir::Ty<'ir>) {
    match &ty.kind {
        ir::TyKind::Fn(params, ret) => {
            params.iter().for_each(|ty| v.visit_ty(ty));
            if let Some(ty) = ret {
                v.visit_ty(ty);
            }
            v.visit_ty(ty);
        }
        ir::TyKind::Box(_, ty) | ir::TyKind::Ptr(ty) | ir::TyKind::Array(ty) => v.visit_ty(ty),
        ir::TyKind::Path(path) => v.visit_path(path),
        ir::TyKind::Tuple(tys) => tys.iter().for_each(|ty| v.visit_ty(ty)),
        ir::TyKind::Infer => {}
    }
}

pub fn walk_let<'ir, V: Visitor<'ir>>(v: &mut V, l: &'ir ir::Let<'ir>) {
    l.init.iter().for_each(|expr| v.visit_expr(expr));
    v.visit_pat(l.pat);
    l.ty.iter().for_each(|ty| v.visit_ty(ty));
}

pub fn walk_path<'ir, V: Visitor<'ir>>(v: &mut V, path: &'ir ir::Path<'ir>) {
    path.segments.iter().for_each(|seg| v.visit_path_segment(seg));
}

pub fn walk_path_segment<'ir, V: Visitor<'ir>>(v: &mut V, segment: &'ir ir::PathSegment<'ir>) {
    v.visit_ident(segment.ident);
    // TODO visit segment.args
}

pub fn walk_pat<'ir, V: Visitor<'ir>>(v: &mut V, pat: &'ir ir::Pattern<'ir>) {
    match &pat.kind {
        ir::PatternKind::Box(pat) => v.visit_pat(pat),
        ir::PatternKind::Struct(path, fields) => {
            v.visit_path(path);
            fields.iter().for_each(|field| {
                v.visit_ident(field.ident);
                v.visit_pat(field.pat);
            });
        }
        ir::PatternKind::Tuple(pats) => pats.iter().for_each(|p| v.visit_pat(p)),
        ir::PatternKind::Lit(expr) => v.visit_expr(expr),
        ir::PatternKind::Binding(ident, subpat, _m) => {
            v.visit_ident(*ident);
            subpat.iter().for_each(|p| v.visit_pat(p));
        }
        ir::PatternKind::Variant(path, pats) => {
            v.visit_path(path);
            pats.iter().for_each(|pat| v.visit_pat(pat));
        }
        ir::PatternKind::Path(path) => v.visit_path(path),
        ir::PatternKind::Wildcard => {}
    }
}
