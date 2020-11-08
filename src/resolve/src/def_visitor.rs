use super::Resolver;
use ast::*;
use ir::{CtorKind, DefKind, HasDefKind, ModuleId, ROOT_MODULE};

/// collects all `DefId`s
/// this forward declares all "hoisted" things such as items & constructors
/// the following things are assigned def_ids
///
/// items
/// impl/assoc items
/// foreign items
/// type parameters (in generics)
/// variants
/// fields declarations
pub struct DefVisitor<'a, 'r> {
    resolver: &'a mut Resolver<'r>,
    curr_mod: ModuleId,
}

impl<'a, 'r> DefVisitor<'a, 'r> {
    pub fn new(resolver: &'a mut Resolver<'r>) -> Self {
        Self { resolver, curr_mod: ROOT_MODULE }
    }

    pub fn with_module<R>(&mut self, module: ModuleId, f: impl FnOnce(&mut Self) -> R) -> R {
        let prev = self.curr_mod;
        self.curr_mod = module;
        let ret = f(self);
        self.curr_mod = prev;
        ret
    }

    pub fn new_module(&mut self, name: Ident) -> ModuleId {
        self.resolver.new_module(self.curr_mod, name)
    }
}

impl<'ast, 'r> Visitor<'ast> for DefVisitor<'ast, 'r> {
    fn visit_item(&mut self, item: &'ast Item) {
        self.resolver.def_item(self.curr_mod, item.ident, item.id, item.kind.def_kind());
        match &item.kind {
            ItemKind::Enum(..) => {
                // enums introduce a new namespace represented as a module
                // where the variants are defined
                let module = self.new_module(item.ident);
                self.with_module(module, |this| ast::walk_item(this, item));
            }
            _ => ast::walk_item(self, item),
        }
    }

    fn visit_foreign_item(&mut self, item: &'ast ForeignItem) {
        self.resolver.def_item(self.curr_mod, item.ident, item.id, item.kind.def_kind());
        ast::walk_foreign_item(self, item);
    }

    fn visit_assoc_item(&mut self, item: &'ast AssocItem) {
        // we allocate a `DefId` for these items,
        // but we do not insert them into the module as are accessed
        // in a type relative path
        self.resolver.define(item.id);
        ast::walk_assoc_item(self, item);
    }

    fn visit_field_decl(&mut self, field: &'ast FieldDecl) {
        self.resolver.define(field.id);
    }

    /// define the variant constructor
    fn visit_variant(&mut self, variant: &'ast Variant) {
        let ctor_kind = match variant.kind {
            VariantKind::Struct(..) => CtorKind::Struct,
            VariantKind::Tuple(..) => CtorKind::Tuple,
            VariantKind::Unit => CtorKind::Unit,
        };
        let def_kind = DefKind::Ctor(ctor_kind);
        self.resolver.def_item(self.curr_mod, variant.ident, variant.id, def_kind);
        ast::walk_variant(self, variant);
    }

    fn visit_ty_param(&mut self, ty_param: &'ast TyParam) {
        self.resolver.define(ty_param.id);
    }
}

impl<'a> Resolver<'a> {
    pub fn resolve_items(&mut self, prog: &Prog) {
        let mut visitor = DefVisitor::new(self);
        visitor.visit_prog(prog);
    }
}
