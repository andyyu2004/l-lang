use crate::TyConv;
use ir::DefId;
use lcore::ty::*;

pub trait Typeof<'tcx> {
    fn ty(&self, tcx: TyCtx<'tcx>, substs: SubstsRef<'tcx>) -> Ty<'tcx>;
}

impl<'tcx> Typeof<'tcx> for FieldTy<'tcx> {
    /// return type of the field
    // we require this indirection instead of storing `ty: Ty` directly as a field
    // because fields may refer to the the struct/enum that it is declared in
    // therefore, the lowering must be done post type collection
    fn ty(&self, tcx: TyCtx<'tcx>, substs: SubstsRef<'tcx>) -> Ty<'tcx> {
        // TODO cache this result somewhere?
        let ty = tcx.ir_ty_to_ty(&self.ir_ty);
        ty.subst(tcx, substs)
    }
}

pub trait TcxTypeofExt<'tcx> {
    fn type_of(self, def_id: DefId) -> Ty<'tcx>;
}

impl<'tcx> TcxTypeofExt<'tcx> for TyCtx<'tcx> {
    fn type_of(self, def_id: DefId) -> Ty<'tcx> {
        type_of(self, def_id)
    }
}

fn type_of<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId) -> Ty<'tcx> {
    let def_node = tcx.defs().get(def_id);
    match def_node {
        ir::DefNode::Item(item) => match item.kind {
            ir::ItemKind::Fn(sig, ..) => tcx.fn_sig_to_ty(sig),
            ir::ItemKind::Enum(..) | ir::ItemKind::Struct(..) => tcx.collected_ty(def_id),
            ir::ItemKind::Impl { generics: _, trait_path: _, self_ty, impl_item_refs: _ } =>
                tcx.ir_ty_to_ty(self_ty),
            _ => unreachable!("unexpected item kind in type_of"),
        },
        ir::DefNode::ImplItem(item) => match item.kind {
            ir::ImplItemKind::Fn(sig, ..) => tcx.fn_sig_to_ty(sig),
        },
        ir::DefNode::ForeignItem(item) => match item.kind {
            ir::ForeignItemKind::Fn(sig, ..) => tcx.fn_sig_to_ty(sig),
        },
        ir::DefNode::Ctor(..) => tcx.collected_ty(def_id),
        ir::DefNode::Variant(..) | ir::DefNode::TyParam(..) => panic!(),
    }
}
