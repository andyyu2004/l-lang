use crate::*;
use ir::{CtorKind, DefKind, HasDefKind};
use lc_ast::*;

/// collects all `DefId`s
/// this forward declares all "hoisted" things such as items & constructors
/// the following things are assigned def_ids
///
/// impl/assoc/trait/foreign items
/// type parameters (in generics)
/// variants and constructors
/// fields declarations
pub struct DefCollector<'a, 'r> {
    resolver: &'a mut Resolver<'r>,
    curr_mod: ModuleId,
}

impl<'a, 'r> DefCollector<'a, 'r> {
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

    pub fn def_module(&mut self, name: Ident) -> ModuleId {
        self.resolver.def_module(self.curr_mod, name)
    }
}

impl<'ast, 'r> Visitor<'ast> for DefCollector<'ast, 'r> {
    fn visit_ast(&mut self, ast: &'ast Ast) {
        let module_id = self.def_module(Ident::empty());
        assert_eq!(module_id, ROOT_MODULE);
        lc_ast::walk_ast(self, ast);
    }

    fn visit_item(&mut self, item: &'ast Item) {
        self.resolver.def_item(self.curr_mod, item.ident, item.id, item.kind.def_kind());
        match &item.kind {
            ItemKind::Enum(..) => {
                // enums introduce a new namespace represented as a module
                // where the variants are defined
                let module = self.def_module(item.ident);
                self.with_module(module, |this| lc_ast::walk_item(this, item));
            }
            ItemKind::Mod(module) => {
                let module_id = self.def_module(item.ident);
                self.with_module(module_id, |this| lc_ast::walk_module(this, module))
            }
            _ => lc_ast::walk_item(self, item),
        }
    }

    fn visit_foreign_item(&mut self, item: &'ast ForeignItem) {
        self.resolver.def_item(self.curr_mod, item.ident, item.id, item.kind.def_kind());
        lc_ast::walk_foreign_item(self, item);
    }

    fn visit_assoc_item(&mut self, item: &'ast AssocItem) {
        // we allocate a `DefId` for these items,
        // but we do not insert them into the module as these are accessed
        // in a type relative path
        self.resolver.define(item.id);
        lc_ast::walk_assoc_item(self, item);
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
        lc_ast::walk_variant(self, variant);
    }

    fn visit_ty_param(&mut self, ty_param: &'ast TyParam) {
        self.resolver.define(ty_param.id);
    }
}

impl<'a> Resolver<'a> {
    pub fn collect_defs(&mut self, prog: &Ast) {
        let mut visitor = DefCollector::new(self);
        visitor.visit_ast(prog);
    }
}
