use super::Resolver;
use crate::ast::{self, *};
use crate::ir::DefKind;

/// collects all `DefId`s
pub struct DefVisitor<'a, 'r> {
    resolver: &'a mut Resolver<'r>,
}

impl<'a, 'r> DefVisitor<'a, 'r> {
    pub fn new(resolver: &'a mut Resolver<'r>) -> Self {
        Self { resolver }
    }
}

impl<'ast, 'r> Visitor<'ast> for DefVisitor<'ast, 'r> {
    fn visit_item(&mut self, item: &'ast Item) {
        self.resolver.def_item(item.ident, item.id, item.kind.def_kind());
        ast::walk_item(self, item);
    }

    fn visit_ty_param(&mut self, ty_param: &'ast TyParam) {
        self.resolver.def(ty_param.ident, ty_param.id);
    }
}

impl<'a> Resolver<'a> {
    pub fn resolve_items(&mut self, prog: &Prog) {
        let mut visitor = DefVisitor::new(self);
        visitor.visit_prog(prog);
    }
}
