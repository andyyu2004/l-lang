use super::Resolver;
use crate::ast::{self, *};
use crate::ir::{CtorKind, DefKind, ModuleId, ROOT_MODULE};

/// collects all `DefId`s
/// this forward declares all "hoisted" things such as items & constructors
pub struct DefVisitor<'a, 'r> {
    resolver: &'a mut Resolver<'r>,
    curr_mod: ModuleId,
    curr_adt_id: Option<NodeId>,
}

impl<'a, 'r> DefVisitor<'a, 'r> {
    pub fn new(resolver: &'a mut Resolver<'r>) -> Self {
        Self { resolver, curr_mod: ROOT_MODULE, curr_adt_id: None }
    }

    pub fn with_module<R>(&mut self, module: ModuleId, f: impl FnOnce(&mut Self) -> R) -> R {
        let prev = self.curr_mod;
        self.curr_mod = module;
        let ret = f(self);
        self.curr_mod = prev;
        ret
    }

    pub fn with_adt<R>(&mut self, id: NodeId, f: impl FnOnce(&mut Self) -> R) -> R {
        let prev = self.curr_adt_id.take();
        self.curr_adt_id = Some(id);
        let ret = f(self);
        self.curr_adt_id = prev;
        ret
    }

    pub fn new_module(&mut self, name: Ident) -> ModuleId {
        self.resolver.new_module(self.curr_mod, name)
    }
}

impl<'ast, 'r> Visitor<'ast> for DefVisitor<'ast, 'r> {
    fn visit_item(&mut self, item: &'ast Item) {
        self.resolver.def_item(self.curr_mod, item.ident, item.id, item.kind.def_kind());
        self.with_adt(item.id, |this| match item.kind {
            ItemKind::Enum(..) => {
                // enums introduce a new namespace represented as a module
                let module = this.new_module(item.ident);
                this.with_module(module, |this| ast::walk_item(this, item));
            }
            _ => ast::walk_item(this, item),
        })
    }

    /// define the variant constructor
    fn visit_variant(&mut self, variant: &'ast Variant) {
        if let VariantKind::Unit = variant.kind {
            self.resolver.def_item(
                self.curr_mod,
                variant.ident,
                variant.id,
                DefKind::Ctor(CtorKind::Unit, self.curr_adt_id.unwrap()),
            );
        } else if let VariantKind::Tuple(_) = variant.kind {
            self.resolver.def_item(
                self.curr_mod,
                variant.ident,
                variant.id,
                DefKind::Ctor(CtorKind::Fn, self.curr_adt_id.unwrap()),
            );
        }
        ast::walk_variant(self, variant);
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
