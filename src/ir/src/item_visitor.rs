use crate::*;
use ast::Ident;

/// visits all items in the `IR`
pub trait ItemVisitor<'ir> {
    fn visit_ir(&mut self, ir: &'ir ir::Ir<'ir>) {
        ir.items.values().for_each(|item| self.visit_item(item));
        ir.impl_items.values().for_each(|item| self.visit_impl_item(item));
    }

    fn visit_item(&mut self, _item: &'ir ir::Item<'ir>) {
    }

    fn visit_impl_item(&mut self, _impl_item: &'ir ir::ImplItem<'ir>) {
    }
}

// this trait allows for a more uniform traversal
// with only one method override needed
pub trait ItemDefVisitor<'ir> {
    fn visit_item_def_id(&mut self, def_id: DefId);

    fn visit_item(&mut self, item: &'ir ir::Item<'ir>) {
        self.visit_item_def_id(item.id.def);
    }

    fn visit_impl_item(&mut self, impl_item: &'ir ir::ImplItem<'ir>) {
        self.visit_item_def_id(impl_item.id.def);
    }
}

// TODO this is pretty bad trait design, redo this when better idea comes to mind
// impl<'ir, V> ItemVisitor<'ir> for V
// where
//     V: FnVisitor<'ir>,
// {
//     fn visit_item(&mut self, item: &'ir ir::Item<'ir>) {
//         FnVisitor::visit_item(self, item)
//     }

//     fn visit_impl_item(&mut self, impl_item: &'ir ir::ImplItem<'ir>) {
//         FnVisitor::visit_impl_item(self, impl_item)
//     }
// }

/// visits all things that requires mir generation (i.e. functions and constructors)
pub trait FnVisitor<'ir> {
    fn visit_ir(&mut self, ir: &'ir ir::Ir<'ir>) {
        ir.items.values().for_each(|item| self.visit_item(item));
        ir.impl_items.values().for_each(|item| self.visit_impl_item(item));
    }

    fn visit_fn(
        &mut self,
        def_id: ir::DefId,
        ident: Ident,
        sig: &'ir ir::FnSig<'ir>,
        generics: &'ir ir::Generics<'ir>,
        body: &'ir ir::Body<'ir>,
    );

    fn visit_foreign_fn(
        &mut self,
        _def_id: DefId,
        _ident: Ident,
        _sig: &'ir FnSig<'ir>,
        _generics: &'ir Generics<'ir>,
    ) {
    }

    fn visit_foreign_item(&mut self, item: &'ir ForeignItem<'ir>) {
        match item.kind {
            ForeignItemKind::Fn(sig, generics) =>
                self.visit_foreign_fn(item.id.def, item.ident, sig, generics),
        }
    }

    fn visit_enum(&mut self, _item: &'ir ir::Item<'ir>) {
    }

    fn visit_item(&mut self, item: &'ir Item<'ir>) {
        match item.kind {
            ir::ItemKind::Fn(sig, generics, body) =>
                self.visit_fn(item.id.def, item.ident, sig, generics, body),
            ir::ItemKind::Enum(..) => self.visit_enum(item),
            ItemKind::Extern(items) => items.iter().for_each(|item| self.visit_foreign_item(item)),
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
