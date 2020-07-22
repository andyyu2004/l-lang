use super::Resolver;
use crate::ast::{Item, Prog, Visitor};

struct ItemResolutionVisitor<'a> {
    resolver: &'a mut Resolver,
}

impl<'a> ItemResolutionVisitor<'a> {
    pub fn new(resolver: &'a mut Resolver) -> Self {
        Self { resolver }
    }
}

impl<'ast> Visitor<'ast> for ItemResolutionVisitor<'ast> {
    fn visit_item(&mut self, item: &'ast Item) {
        self.resolver.def_item(item.ident, item.id, item.kind.def_kind());
    }
}

impl Resolver {
    crate fn resolve_items(&mut self, prog: &Prog) {
        let mut visitor = ItemResolutionVisitor::new(self);
        visitor.visit_prog(prog);
    }
}
