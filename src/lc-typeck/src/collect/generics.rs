use crate::TyConv;
use ir::{DefId, DefNode};
use lc_core::queries::Queries;
use lc_core::ty::{self, TyCtx, TyParam};

pub(crate) fn provide(queries: &mut Queries) {
    *queries = Queries { generics_of, ..*queries }
}

pub fn generics_of<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId) -> &'tcx ty::Generics<'tcx> {
    let generics = tcx.defs().generics(def_id);

    // impl item is the only kind of defnode that may possibly have outer generics
    let impl_generic_params = match tcx.defs().get(def_id) {
        DefNode::ImplItem(item) => generics_of(tcx, item.impl_def_id).params,
        _ => &[],
    };

    let generic_params = tcx.alloc_iter(generics.params.iter().map(
        |&ir::TyParam { id, index, ident, span, default }| TyParam {
            id,
            span,
            ident,
            index,
            default: default.map(|ty| tcx.ir_ty_to_ty(ty)),
        },
    ));

    let mut params = impl_generic_params.to_vec();
    params.extend(generic_params);
    let params = tcx.alloc_iter(params);

    tcx.alloc(ty::Generics { params })
}
