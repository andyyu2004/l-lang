use crate::ast::Ident;
use crate::ir::{self, Visitor};
use crate::mir;
use crate::typeck::TyCtx;

pub struct CodegenCollector<'tcx> {
    tcx: TyCtx<'tcx>,
    items: Vec<CodegenItem<'tcx>>,
}

pub struct CodegenItem<'tcx> {
    ident: Ident,
    body: &'tcx mir::Body<'tcx>,
}

impl<'tcx> CodegenItem<'tcx> {
}

impl<'tcx> Visitor<'tcx> for CodegenCollector<'tcx> {
    fn visit_item(&mut self, item: &'tcx ir::Item<'tcx>) {
        match item.kind {
            ir::ItemKind::Fn(_, _, body) => todo!(),
            ir::ItemKind::Struct(..) => {}
            ir::ItemKind::Enum(..) => {}
            ir::ItemKind::Impl { generics, trait_path, self_ty, impl_item_refs } => {}
        }
    }

    fn visit_impl_item(&mut self, item: &'tcx ir::ImplItem<'tcx>) {
        match item.kind {
            ir::ImplItemKind::Fn(_, body) => todo!(),
        }
    }
}
