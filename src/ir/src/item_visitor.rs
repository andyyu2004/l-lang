use crate as ir;
use ast::Ident;

/// visits all items in the `IR`
pub trait ItemVisitor<'ir> {
    fn visit_ir(&mut self, ir: &'ir ir::IR<'ir>) {
        ir.items.values().for_each(|item| self.visit_item(item));
        ir.impl_items.values().for_each(|item| self.visit_impl_item(item));
    }

    fn visit_item(&mut self, _item: &'ir ir::Item<'ir>) {
    }

    fn visit_impl_item(&mut self, _impl_item: &'ir ir::ImplItem<'ir>) {
    }
}

impl<'ir, V> ItemVisitor<'ir> for V
where
    V: FnVisitor<'ir>,
{
    fn visit_item(&mut self, item: &'ir ir::Item<'ir>) {
        FnVisitor::visit_item(self, item)
    }

    fn visit_impl_item(&mut self, impl_item: &'ir ir::ImplItem<'ir>) {
        FnVisitor::visit_impl_item(self, impl_item)
    }
}

/// visits all things that requires mir generation (i.e. functions and constructors)
pub trait FnVisitor<'ir>: ItemVisitor<'ir> {
    fn visit_fn(
        &mut self,
        def_id: ir::DefId,
        ident: Ident,
        sig: &'ir ir::FnSig<'ir>,
        generics: &'ir ir::Generics<'ir>,
        body: &'ir ir::Body<'ir>,
    );

    fn visit_enum(&mut self, _item: &'ir ir::Item<'ir>) {
    }

    fn visit_item(&mut self, item: &'ir ir::Item<'ir>) {
        match item.kind {
            ir::ItemKind::Fn(sig, generics, body) =>
                self.visit_fn(item.id.def, item.ident, sig, generics, body),
            ir::ItemKind::Enum(..) => self.visit_enum(item),
            ir::ItemKind::Extern(..) => todo!(),
            ir::ItemKind::Struct(..) => {}
            ir::ItemKind::Impl { .. } => {}
        }
    }

    fn visit_impl_item(&mut self, impl_item: &'ir ir::ImplItem<'ir>) {
        match impl_item.kind {
            ir::ImplItemKind::Fn(sig, body) =>
                self.visit_fn(impl_item.id.def, impl_item.ident, sig, impl_item.generics, body),
        }
    }
}
