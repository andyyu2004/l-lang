use ir::{DefId, Visitor};

pub struct Validator<'ir> {
    pd: std::marker::PhantomData<&'ir ()>,
    curr_def_id: Option<DefId>,
}

// refer to `resolve/src/def_collector.rs` to see what creates a new def_id
impl<'ir> Validator<'ir> {
    pub fn new() -> Self {
        Self { curr_def_id: None, pd: std::marker::PhantomData }
    }

    fn with_def_id<R>(&mut self, def_id: DefId, f: impl FnOnce(&mut Self) -> R) -> R {
        let prev = self.curr_def_id.take();
        self.curr_def_id = Some(def_id);
        let ret = f(self);
        self.curr_def_id = prev;
        ret
    }
}

impl<'ir> Visitor<'ir> for Validator<'ir> {
    fn visit_item(&mut self, item: &'ir ir::Item<'ir>) {
        dbg!(item.id.def);
        self.with_def_id(item.id.def, |this| ir::walk_item(this, item))
    }

    fn visit_impl_item(&mut self, impl_item: &'ir ir::ImplItem<'ir>) {
        self.with_def_id(impl_item.id.def, |this| ir::walk_impl_item(this, impl_item))
    }

    fn visit_foreign_item(&mut self, foreign_item: &'ir ir::ForeignItem<'ir>) {
        self.with_def_id(foreign_item.id.def, |this| ir::walk_foreign_item(this, foreign_item));
    }

    fn visit_ty_param(&mut self, param: &'ir ir::TyParam<'ir>) {
        self.with_def_id(param.id.def, |this| ir::walk_ty_param(this, param))
    }

    fn visit_variant(&mut self, variant: &'ir ir::Variant<'ir>) {
        self.with_def_id(variant.id.def, |this| ir::walk_variant(this, variant))
    }

    fn visit_field_decl(&mut self, decl: &'ir ir::FieldDecl<'ir>) {
        self.with_def_id(decl.id.def, |this| ir::walk_field_decl(this, decl));
    }

    fn visit_id(&mut self, ir: ir::Id) {
        assert_eq!(self.curr_def_id.unwrap(), ir.def);
    }
}
