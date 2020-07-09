use super::Resolver;
use crate::ast::Prog;

struct LateResolutionVisitor<'r> {
    resolver: &'r mut Resolver,
}

impl<'r> LateResolutionVisitor<'r> {
    pub fn new(resolver: &'r mut Resolver) -> Self {
        Self { resolver }
    }
}

impl Resolver {
    crate fn late_resolve_prog(&mut self, prog: &Prog) {
        let visitor = LateResolutionVisitor::new(self);
    }
}
