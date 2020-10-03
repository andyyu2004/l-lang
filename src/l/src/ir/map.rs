use crate::ir::{self, DefId};

impl<'ir> ir::IR<'ir> {
    pub fn fn_info(
        &self,
        def_id: DefId,
    ) -> (&'ir ir::FnSig<'ir>, &'ir ir::Generics<'ir>, &'ir ir::Body<'ir>) {
        if let Some(item) = self.items.get(&def_id) {
            match item.kind {
                ir::ItemKind::Fn(sig, generics, body) => (sig, generics, body),
                _ => unreachable!(),
            }
        } else if let Some(impl_item) = self.impl_items.get(&ir::ImplItemId(def_id)) {
            match impl_item.kind {
                ir::ImplItemKind::Fn(sig, body) => (sig, impl_item.generics, body),
            }
        } else {
            // TODO check for trait items too
            panic!()
        }
    }
}
