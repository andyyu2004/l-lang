use super::*;

/// traverse the ast; each function can be overridden.
/// by default, just recursively visits each substructure
pub trait Visitor<'ast>: Sized {
    fn visit_item(&mut self, item: &'ast Item) {
        walk_item(self, item)
    }

    fn visit_foreign_item(&mut self, item: &'ast ForeignItem) {
        walk_foreign_item(self, item);
    }

    fn visit_prog(&mut self, prog: &'ast Prog) {
        prog.items.iter().for_each(|item| self.visit_item(item));
    }

    fn visit_variant(&mut self, variant: &'ast Variant) {
        walk_variant(self, variant);
    }

    fn visit_variant_kind(&mut self, kind: &'ast VariantKind) {
        walk_variant_kind(self, kind);
    }

    fn visit_generics(&mut self, generics: &'ast Generics) {
        walk_generics(self, generics)
    }

    fn visit_ty_param(&mut self, ty_param: &'ast TyParam) {
        self.visit_ident(ty_param.ident)
    }

    fn visit_vis(&mut self, _vis: &'ast Visibility) {
    }

    fn visit_fn(&mut self, sig: &'ast FnSig, body: Option<&'ast Expr>) {
        walk_fn(self, sig, body);
    }

    fn visit_block(&mut self, block: &'ast Block) {
        walk_block(self, block);
    }

    /// visit the initializer first in case the same pattern is referenced in the initializer
    /// let x = 1;
    /// let x = x;
    /// this will only behave correctly if the pattern is resolved after the initializer
    fn visit_let(&mut self, l: &'ast Let) {
        walk_let(self, l);
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        walk_expr(self, expr)
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        walk_stmt(self, stmt);
    }

    fn visit_fn_sig(&mut self, sig: &'ast FnSig) {
        walk_fn_sig(self, sig);
    }

    fn visit_closure(&mut self, name: Option<Ident>, sig: &'ast FnSig, expr: &'ast Expr) {
        walk_closure(self, name, sig, expr)
    }

    fn visit_param(&mut self, param: &'ast Param) {
        self.visit_pattern(&param.pattern);
        self.visit_ty(&param.ty);
    }

    fn visit_field_decl(&mut self, field: &'ast FieldDecl) {
        walk_field_decl(self, field)
    }

    fn visit_field(&mut self, field: &'ast Field) {
        walk_field(self, field)
    }

    fn visit_pattern(&mut self, pattern: &'ast Pattern) {
        walk_pat(self, pattern);
    }

    fn visit_path(&mut self, path: &'ast Path) {
        walk_path(self, path);
    }

    fn visit_path_segment(&mut self, segment: &'ast PathSegment) {
        walk_path_segment(self, segment);
    }

    fn visit_generic_args(&mut self, args: &'ast GenericArgs) {
        walk_generic_args(self, args)
    }

    fn visit_ty(&mut self, ty: &'ast Ty) {
        walk_ty(self, ty)
    }

    fn visit_ident(&mut self, _ident: Ident) {
    }

    fn visit_arm(&mut self, arm: &'ast Arm) {
        walk_arm(self, arm);
    }

    fn visit_assoc_item(&mut self, item: &'ast AssocItem) {
        walk_assoc_item(self, item)
    }
}

pub fn walk_fn_sig<'ast>(visitor: &mut impl Visitor<'ast>, sig: &'ast FnSig) {
    sig.params.iter().for_each(|param| visitor.visit_param(param));
    sig.ret_ty.iter().for_each(|ty| visitor.visit_ty(ty));
}

pub fn walk_foreign_item<'ast>(visitor: &mut impl Visitor<'ast>, item: &'ast ForeignItem) {
    match &item.kind {
        ForeignItemKind::Fn(sig, generics) => {
            visitor.visit_fn_sig(sig);
            visitor.visit_generics(generics);
        }
    }
}

fn walk_generic_args<'ast>(visitor: &mut impl Visitor<'ast>, args: &'ast GenericArgs) {
    args.args.iter().for_each(|ty| visitor.visit_ty(ty));
}

pub fn walk_fn<'ast>(visitor: &mut impl Visitor<'ast>, sig: &'ast FnSig, body: Option<&'ast Expr>) {
    visitor.visit_fn_sig(sig);
    body.iter().for_each(|body| visitor.visit_expr(body));
}

pub fn walk_block<'ast>(visitor: &mut impl Visitor<'ast>, block: &'ast Block) {
    block.stmts.iter().for_each(|stmt| visitor.visit_stmt(stmt))
}

pub fn walk_stmt<'ast>(visitor: &mut impl Visitor<'ast>, stmt: &'ast Stmt) {
    match &stmt.kind {
        StmtKind::Let(l) => visitor.visit_let(l),
        StmtKind::Expr(expr) => visitor.visit_expr(expr),
        StmtKind::Semi(expr) => visitor.visit_expr(expr),
    }
}

/// visit the initializer first in case the same pattern is referenced in the initializer
/// let x = 1;
/// let x = x;
/// this will only behave correctly if the pattern is resolved after the initializer
pub fn walk_let<'ast>(visitor: &mut impl Visitor<'ast>, Let { pat, ty, init, .. }: &'ast Let) {
    init.iter().for_each(|expr| visitor.visit_expr(expr));
    visitor.visit_pattern(pat);
    ty.iter().for_each(|ty| visitor.visit_ty(ty));
}

pub fn walk_path<'ast>(visitor: &mut impl Visitor<'ast>, path: &'ast Path) {
    path.segments.iter().for_each(|seg| visitor.visit_path_segment(seg));
}

pub fn walk_path_segment<'ast>(visitor: &mut impl Visitor<'ast>, segment: &'ast PathSegment) {
    visitor.visit_ident(segment.ident);
    segment.args.iter().for_each(|args| visitor.visit_generic_args(args));
}

pub fn walk_arm<'ast>(visitor: &mut impl Visitor<'ast>, arm: &'ast Arm) {
    visitor.visit_pattern(&arm.pat);
    visitor.visit_expr(&arm.body);
    arm.guard.iter().for_each(|expr| visitor.visit_expr(expr));
}

pub fn walk_expr<'ast>(visitor: &mut impl Visitor<'ast>, expr: &'ast Expr) {
    match &expr.kind {
        ExprKind::Lit(_) => {}
        ExprKind::Ret(expr) => expr.iter().for_each(|expr| visitor.visit_expr(expr)),
        ExprKind::Unary(_, expr) => visitor.visit_expr(expr),
        ExprKind::Paren(expr) => visitor.visit_expr(expr),
        ExprKind::Block(block) => visitor.visit_block(block),
        ExprKind::Path(path) => visitor.visit_path(path),
        ExprKind::Tuple(xs) => xs.iter().for_each(|expr| visitor.visit_expr(expr)),
        ExprKind::Closure(name, sig, expr) => visitor.visit_closure(*name, sig, expr),
        ExprKind::Box(expr) => visitor.visit_expr(expr),
        ExprKind::Assign(l, r) => {
            visitor.visit_expr(l);
            visitor.visit_expr(r);
        }
        ExprKind::Struct(path, fields) => {
            visitor.visit_path(path);
            fields.iter().for_each(|f| visitor.visit_field(f));
        }
        ExprKind::Call(f, args) => {
            visitor.visit_expr(f);
            args.iter().for_each(|expr| visitor.visit_expr(expr));
        }
        ExprKind::If(c, l, r) => {
            visitor.visit_expr(c);
            visitor.visit_block(l);
            r.iter().for_each(|expr| visitor.visit_expr(expr));
        }
        ExprKind::Bin(_, l, r) => {
            visitor.visit_expr(l);
            visitor.visit_expr(r);
        }
        ExprKind::Field(expr, ident) => {
            visitor.visit_expr(expr);
            visitor.visit_ident(*ident);
        }
        ExprKind::Match(expr, arms) => {
            visitor.visit_expr(expr);
            arms.iter().for_each(|arm| visitor.visit_arm(arm));
        }
    }
}

pub fn walk_generics<'ast>(visitor: &mut impl Visitor<'ast>, generics: &'ast Generics) {
    generics.params.iter().for_each(|p| visitor.visit_ty_param(p));
}

pub fn walk_closure<'ast>(
    visitor: &mut impl Visitor<'ast>,
    name: Option<Ident>,
    sig: &'ast FnSig,
    expr: &'ast Expr,
) {
    name.map(|ident| visitor.visit_ident(ident));
    visitor.visit_fn_sig(sig);
    visitor.visit_expr(expr);
}

pub fn walk_pat<'ast>(visitor: &mut impl Visitor<'ast>, pat: &'ast Pattern) {
    match &pat.kind {
        PatternKind::Wildcard => {}
        PatternKind::Paren(pat) => visitor.visit_pattern(pat),
        PatternKind::Path(path) => visitor.visit_path(path),
        PatternKind::Tuple(pats) => pats.iter().for_each(|p| visitor.visit_pattern(p)),
        PatternKind::Ident(ident, pat, _) => {
            visitor.visit_ident(*ident);
            pat.iter().for_each(|p| visitor.visit_pattern(p));
        }
        PatternKind::Variant(path, pats) => {
            visitor.visit_path(path);
            pats.iter().for_each(|p| visitor.visit_pattern(p));
        }
        PatternKind::Lit(expr) => visitor.visit_expr(expr),
    }
}

pub fn walk_ty<'ast>(visitor: &mut impl Visitor<'ast>, ty: &'ast Ty) {
    match &ty.kind {
        TyKind::Array(ty) => visitor.visit_ty(ty),
        TyKind::Tuple(tys) => tys.iter().for_each(|ty| visitor.visit_ty(ty)),
        TyKind::Paren(ty) => visitor.visit_ty(ty),
        TyKind::Path(path) => visitor.visit_path(path),
        TyKind::Fn(params, ret) => {
            params.iter().for_each(|ty| visitor.visit_ty(ty));
            ret.iter().for_each(|ty| visitor.visit_ty(ty));
        }
        TyKind::Infer => {}
        TyKind::Ptr(_, ty) => visitor.visit_ty(ty),
    }
}

pub fn walk_field<'ast>(visitor: &mut impl Visitor<'ast>, field: &'ast Field) {
    visitor.visit_expr(&field.expr);
    visitor.visit_ident(field.ident);
}

pub fn walk_field_decl<'ast>(visitor: &mut impl Visitor<'ast>, field: &'ast FieldDecl) {
    field.ident.map(|ident| visitor.visit_ident(ident));
    visitor.visit_vis(&field.vis);
    visitor.visit_ty(&field.ty);
}

pub fn walk_variant<'ast>(visitor: &mut impl Visitor<'ast>, variant: &'ast Variant) {
    visitor.visit_ident(variant.ident);
    visitor.visit_variant_kind(&variant.kind);
}

pub fn walk_variant_kind<'ast>(visitor: &mut impl Visitor<'ast>, kind: &'ast VariantKind) {
    match kind {
        VariantKind::Struct(fields) | VariantKind::Tuple(fields) =>
            fields.iter().for_each(|f| visitor.visit_field_decl(f)),
        VariantKind::Unit => {}
    }
}

pub fn walk_assoc_item<'ast>(visitor: &mut impl Visitor<'ast>, item: &'ast AssocItem) {
    let Item { vis, ident, kind, .. } = &item;
    visitor.visit_vis(vis);
    visitor.visit_ident(*ident);
    match kind {
        AssocItemKind::Fn(sig, generics, body) => {
            visitor.visit_generics(generics);
            visitor.visit_fn(sig, body.as_deref());
        }
    }
}

pub fn walk_item<'ast>(visitor: &mut impl Visitor<'ast>, item: &'ast Item) {
    let Item { vis, ident, kind, .. } = &item;
    visitor.visit_vis(vis);
    visitor.visit_ident(*ident);
    match kind {
        ItemKind::Fn(sig, generics, body) => {
            visitor.visit_generics(generics);
            visitor.visit_fn(sig, body.as_deref())
        }
        ItemKind::Enum(generics, variants) => {
            visitor.visit_generics(generics);
            variants.iter().for_each(|variant| visitor.visit_variant(variant));
        }
        ItemKind::Struct(generics, variant_kind) => {
            visitor.visit_generics(generics);
            visitor.visit_variant_kind(variant_kind);
        }
        ItemKind::Impl { generics, trait_path, self_ty, items } => {
            visitor.visit_generics(generics);
            trait_path.iter().for_each(|path| visitor.visit_path(path));
            visitor.visit_ty(self_ty);
            items.iter().for_each(|item| visitor.visit_assoc_item(item));
        }
        ItemKind::Extern(items) => items.iter().for_each(|item| visitor.visit_foreign_item(item)),
    }
}
