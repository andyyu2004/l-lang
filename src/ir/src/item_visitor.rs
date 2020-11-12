use crate::*;

/// visits all items in the `IR`
pub trait ItemVisitor<'ir> {
    fn visit_item(&mut self, _item: &'ir ir::Item<'ir>);
    fn visit_impl_item(&mut self, _impl_item: &'ir ir::ImplItem<'ir>);
    fn visit_trait_item(&mut self, _trait_item: &'ir ir::TraitItem<'ir>);

    fn visit_ir(&mut self, ir: &'ir ir::Ir<'ir>) {
        ir.items.values().for_each(|item| self.visit_item(item));
        ir.impl_items.values().for_each(|impl_item| self.visit_impl_item(impl_item));
        ir.trait_items.values().for_each(|trait_item| self.visit_trait_item(trait_item));
    }
}

/// visits the def_id of all (non-foreign) function items
/// this includes all the things that have the following properties:
/// - has corresponding mir
/// - requires typeck
///
pub trait FnVisitor<'ir> {
    fn visit_fn(&mut self, def_id: DefId);
}

impl<'ir, V> ItemVisitor<'ir> for V
where
    V: FnVisitor<'ir>,
{
    fn visit_item(&mut self, item: &'ir ir::Item<'ir>) {
        match item.kind {
            ir::ItemKind::Fn(..) => self.visit_fn(item.id.def),
            ir::ItemKind::Extern(..)
            | ir::ItemKind::TypeAlias(..)
            | ir::ItemKind::Enum(..)
            | ir::ItemKind::Struct(..)
            | ir::ItemKind::Impl { .. } => {}
        }
    }

    fn visit_impl_item(&mut self, impl_item: &'ir ir::ImplItem<'ir>) {
        match impl_item.kind {
            ir::ImplItemKind::Fn(..) => self.visit_fn(impl_item.id.def),
        }
    }

    fn visit_trait_item(&mut self, _trait_item: &'ir ir::TraitItem<'ir>) {
        todo!()
    }
}
