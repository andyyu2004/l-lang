use crate::*;

/// visits all items in the `IR`
pub trait ItemVisitor<'ir> {
    fn visit_ir(&mut self, ir: &'ir ir::Ir<'ir>) {
        ir.items.values().for_each(|item| self.visit_item(item));
        ir.impl_items.values().for_each(|item| self.visit_impl_item(item));
    }

    fn visit_item(&mut self, _item: &'ir ir::Item<'ir>);
    fn visit_impl_item(&mut self, _impl_item: &'ir ir::ImplItem<'ir>);
    fn visit_trait_item(&mut self, _trait_item: &'ir ir::TraitItem<'ir>);
}

/// visits the def_id of all (non-foreign) function items
pub trait FnVisitor<'ir> {
    fn visit_fn(&mut self, def_id: DefId);

    fn visit_ir(&mut self, ir: &'ir ir::Ir<'ir>) {
        ir.items.values().for_each(|item| self.visit_item(item));
        ir.impl_items.values().for_each(|item| self.visit_impl_item(item));
        ir.trait_items.values().for_each(|item| self.visit_trait_item(item));
    }

    fn visit_foreign_item(&mut self, item: &'ir ForeignItem<'ir>) {
        match item.kind {
            ForeignItemKind::Fn(..) => self.visit_fn(item.id.def),
        }
    }

    fn visit_item(&mut self, item: &'ir Item<'ir>) {
        match item.kind {
            ir::ItemKind::Fn(..) => self.visit_fn(item.id.def),
            ir::ItemKind::Extern(..)
            | ir::ItemKind::Enum(..)
            | ir::ItemKind::Struct(..)
            | ir::ItemKind::Impl { .. } => {}
        }
    }

    fn visit_trait_item(&mut self, _trait_item: &'ir ir::TraitItem<'ir>) {
        todo!()
    }

    fn visit_impl_item(&mut self, impl_item: &'ir ir::ImplItem<'ir>) {
        match impl_item.kind {
            ir::ImplItemKind::Fn(..) => self.visit_fn(impl_item.id.def),
        }
    }
}
