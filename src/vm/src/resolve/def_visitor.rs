use super::Resolver;
use crate::ast::{self, *};
use crate::ir::DefKind;

/// collects all `DefId`s
crate struct DefVisitor<'a> {
    resolver: &'a mut Resolver,
}

impl<'a> DefVisitor<'a> {
    pub fn new(resolver: &'a mut Resolver) -> Self {
        Self { resolver }
    }
}

impl<'ast> Visitor<'ast> for DefVisitor<'ast> {
    fn visit_item(&mut self, item: &'ast Item) {
        self.resolver.def_item(item.ident, item.id, item.kind.def_kind());
        ast::walk_item(self, item);
    }

    fn visit_ty_param(&mut self, ty_param: &'ast TyParam) {
        self.resolver.def(ty_param.ident, ty_param.id);
    }
}

impl Resolver {
    crate fn resolve_items(&mut self, prog: &Prog) {
        let mut visitor = DefVisitor::new(self);
        visitor.visit_prog(prog);
    }
}
