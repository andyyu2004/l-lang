use crate::collect_item_types;
use crate::typeck_fn;
use lcore::TyCtx;
use std::collections::BTreeMap;

// ir -> tir
// this isn't actually used in the compiler pipeline anymore, its mostly for testing and debugging
// some older tests rely on this
pub fn build_tir<'tcx>(tcx: TyCtx<'tcx>) -> tir::Prog<'tcx> {
    collect_item_types(tcx);
    let prog = tcx.ir;
    let mut items = BTreeMap::new();

    for item in prog.items.values() {
        match item.kind {
            ir::ItemKind::Fn(sig, generics, body) => {
                if let Ok(tir) = typeck_fn(tcx, item.id.def, sig, generics, body, |mut lctx| {
                    lctx.lower_item_tir(item)
                }) {
                    items.insert(item.id, tir);
                }
            }
            ir::ItemKind::Struct(..) => {}
            // note that no tir is generated for enum constructors
            // the constructor code is generated at mir level only
            ir::ItemKind::Enum(..) => {}
            ir::ItemKind::Impl { generics, trait_path, self_ty, impl_item_refs } =>
                unimplemented!(),
        }
    }
    tir::Prog { items }
}
