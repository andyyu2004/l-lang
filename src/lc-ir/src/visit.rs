use crate::*;
use lc_ast::{Ident, Visibility};

pub trait Visitor<'ir>: Sized {
    fn visit_ir(&mut self, ir: &'ir Ir<'ir>) {
        walk_ir(self, ir)
    }

    fn visit_item(&mut self, item: &'ir ir::Item<'ir>) {
        walk_item(self, item)
    }

    fn visit_id(&mut self, _id: ir::Id) {
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

    fn visit_qpath(&mut self, qpath: &'ir ir::QPath<'ir>) {
        walk_qpath(self, qpath)
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

    fn visit_trait_item(&mut self, trait_item: &'ir ir::TraitItem<'ir>) {
        walk_trait_item(self, trait_item);
    }
}

pub fn walk_ir<'ir>(v: &mut impl Visitor<'ir>, ir: &'ir ir::Ir<'ir>) {
    ir.items.values().for_each(|item| v.visit_item(item));
    ir.impl_items.values().for_each(|impl_item| v.visit_impl_item(impl_item));
    ir.trait_items.values().for_each(|trait_item| v.visit_trait_item(trait_item));
}

pub fn walk_trait_item<'ir>(v: &mut impl Visitor<'ir>, trait_item: &'ir ir::TraitItem<'ir>) {
    v.visit_id(trait_item.id);
    v.visit_ident(trait_item.ident);
    v.visit_vis(trait_item.vis);
    match trait_item.kind {
        ir::TraitItemKind::Fn(sig, body) => {
            v.visit_fn_sig(sig);
            body.iter().for_each(|body| v.visit_body(body));
        }
    }
}

pub fn walk_impl_item<'ir>(v: &mut impl Visitor<'ir>, impl_item: &'ir ir::ImplItem<'ir>) {
    v.visit_id(impl_item.id);
    v.visit_ident(impl_item.ident);
    v.visit_vis(impl_item.vis);
    match impl_item.kind {
        ir::ImplItemKind::Fn(sig, body) => {
            v.visit_fn_sig(sig);
            v.visit_body(body);
        }
    }
}

pub fn walk_foreign_item<'ir>(v: &mut impl Visitor<'ir>, item: &'ir ForeignItem<'ir>) {
    v.visit_id(item.id);
    v.visit_ident(item.ident);
    v.visit_vis(item.vis);
    match item.kind {
        ForeignItemKind::Fn(sig, generics) => {
            v.visit_fn_sig(sig);
            v.visit_generics(generics);
        }
    }
}

pub fn walk_variant<'ir>(v: &mut impl Visitor<'ir>, variant: &'ir ir::Variant<'ir>) {
    v.visit_id(variant.id);
    v.visit_ident(variant.ident);
    v.visit_variant_kind(&variant.kind);
}

pub fn walk_field_decl<'ir>(v: &mut impl Visitor<'ir>, decl: &'ir ir::FieldDecl<'ir>) {
    v.visit_id(decl.id);
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
    v.visit_id(param.id);
    param.default.iter().for_each(|ty| v.visit_ty(ty));
}

pub fn walk_generics<'ir>(v: &mut impl Visitor<'ir>, generics: &'ir ir::Generics<'ir>) {
    generics.params.iter().for_each(|param| v.visit_ty_param(param));
}

pub fn walk_fn_sig<'ir, V: Visitor<'ir>>(v: &mut V, sig: &'ir ir::FnSig<'ir>) {
    sig.inputs.iter().for_each(|ty| v.visit_ty(ty));
    sig.output.iter().for_each(|ty| v.visit_ty(ty));
}

pub fn walk_qpath<'ir>(v: &mut impl Visitor<'ir>, qpath: &'ir ir::QPath<'ir>) {
    match qpath {
        ir::QPath::Resolved(path) => v.visit_path(path),
        ir::QPath::TypeRelative(ty, segment) => {
            v.visit_ty(ty);
            v.visit_path_segment(segment);
        }
    }
}

pub fn walk_item<'ir, V: Visitor<'ir>>(v: &mut V, item: &'ir ir::Item<'ir>) {
    v.visit_id(item.id);
    v.visit_vis(item.vis);
    v.visit_ident(item.ident);
    match &item.kind {
        ir::ItemKind::Fn(sig, generics, body) => {
            v.visit_fn_sig(sig);
            v.visit_generics(generics);
            v.visit_body(body);
        }
        ir::ItemKind::Use(path) => v.visit_path(path),
        ir::ItemKind::TypeAlias(generics, ty) => {
            v.visit_generics(generics);
            v.visit_ty(ty);
        }
        ir::ItemKind::Enum(generics, variants) => {
            v.visit_generics(generics);
            variants.iter().for_each(|variant| v.visit_variant(variant));
        }
        ir::ItemKind::Struct(generics, kind) => {
            v.visit_generics(generics);
            v.visit_variant_kind(kind);
        }
        ir::ItemKind::Extern(_abi, items) =>
            items.iter().for_each(|item| v.visit_foreign_item(item)),
        // currently, the top level `walk_ir` just walks through the items, impl_items, and
        // trait_items map
        // this doesn't visit things in proper nested order but maybe it is fine?
        // this is why modules, traits, and impls don't need to recursively walk
        ir::ItemKind::Impl { generics, trait_path, self_ty, impl_item_refs: _ } => {
            v.visit_generics(generics);
            trait_path.iter().for_each(|path| v.visit_path(path));
            v.visit_ty(self_ty);
        }
        ir::ItemKind::Trait { generics, trait_item_refs: _ } => v.visit_generics(generics),
        ir::ItemKind::Mod(..) => {}
    }
}

pub fn walk_body<'ir, V: Visitor<'ir>>(v: &mut V, body: &'ir ir::Body<'ir>) {
    body.params.iter().for_each(|p| v.visit_param(p));
    v.visit_expr(body.expr)
}

pub fn walk_param<'ir, V: Visitor<'ir>>(v: &mut V, param: &'ir ir::Param<'ir>) {
    v.visit_id(param.id);
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
    v.visit_id(expr.id);
    match &expr.kind {
        ir::ExprKind::Box(expr) => v.visit_expr(expr),
        ir::ExprKind::Loop(block) => v.visit_block(block),
        ir::ExprKind::Unary(_, expr) => v.visit_expr(expr),
        ir::ExprKind::Block(block) => v.visit_block(block),
        ir::ExprKind::Path(qpath) => v.visit_qpath(qpath),
        ir::ExprKind::Tuple(xs) => xs.iter().for_each(|x| v.visit_expr(x)),
        ir::ExprKind::Closure(sig, body) => v.visit_lambda(sig, body),
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
        ir::ExprKind::Struct(qpath, fields) => {
            v.visit_qpath(qpath);
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
        ir::ExprKind::Err | ir::ExprKind::Break | ir::ExprKind::Continue => {}
    }
}

pub fn walk_field<'ir, V: Visitor<'ir>>(v: &mut V, field: &'ir ir::Field<'ir>) {
    v.visit_id(field.id);
    v.visit_ident(field.ident);
    v.visit_expr(field.expr);
}

pub fn walk_arm<'ir, V: Visitor<'ir>>(v: &mut V, arm: &'ir ir::Arm<'ir>) {
    v.visit_id(arm.id);
    v.visit_pat(arm.pat);
    arm.guard.iter().for_each(|expr| v.visit_expr(expr));
    v.visit_expr(arm.body);
}

pub fn walk_stmt<'ir, V: Visitor<'ir>>(v: &mut V, stmt: &'ir ir::Stmt<'ir>) {
    v.visit_id(stmt.id);
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
    v.visit_id(ty.id);
    match &ty.kind {
        ir::TyKind::Fn(params, ret) => {
            params.iter().for_each(|ty| v.visit_ty(ty));
            if let Some(ty) = ret {
                v.visit_ty(ty);
            }
            v.visit_ty(ty);
        }
        ir::TyKind::Box(ty) | ir::TyKind::Ptr(ty) | ir::TyKind::Array(ty) => v.visit_ty(ty),
        ir::TyKind::Path(qpath) => v.visit_qpath(qpath),
        ir::TyKind::Tuple(tys) => tys.iter().for_each(|ty| v.visit_ty(ty)),
        ir::TyKind::Err | ir::TyKind::Infer => {}
    }
}

pub fn walk_let<'ir, V: Visitor<'ir>>(v: &mut V, l: &'ir ir::Let<'ir>) {
    v.visit_id(l.id);
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
    v.visit_id(pat.id);
    match &pat.kind {
        ir::PatternKind::Box(pat) => v.visit_pat(pat),
        ir::PatternKind::Struct(qpath, fields) => {
            v.visit_qpath(qpath);
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
        ir::PatternKind::Variant(qpath, pats) => {
            v.visit_qpath(qpath);
            pats.iter().for_each(|pat| v.visit_pat(pat));
        }
        ir::PatternKind::Path(qpath) => v.visit_qpath(qpath),
        ir::PatternKind::Wildcard => {}
    }
}
