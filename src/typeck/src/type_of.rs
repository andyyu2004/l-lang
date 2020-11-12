use crate::TyConv;
use ir::{DefId, DefNode};
use lcore::queries::Queries;
use lcore::ty::*;

pub fn provide(queries: &mut Queries) {
    *queries = Queries { type_of, fn_sig, ..*queries }
}

fn type_of<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId) -> Ty<'tcx> {
    let def_node = tcx.defs().get(def_id);
    match def_node {
        ir::DefNode::Item(item) => match item.kind {
            ir::ItemKind::Fn(..) => tcx.mk_fn_ptr(tcx.fn_sig(def_id)),
            ir::ItemKind::Enum(..) | ir::ItemKind::Struct(..) => self::type_of_adt(tcx, def_id),
            ir::ItemKind::Impl { generics: _, trait_path: _, self_ty, impl_item_refs: _ } =>
                tcx.ir_ty_to_ty(self_ty),
            ir::ItemKind::TypeAlias(_, ty) => tcx.ir_ty_to_ty(ty),
            ir::ItemKind::Use(..) | ir::ItemKind::Extern(..) => panic!(),
        },
        ir::DefNode::Ctor(variant) | ir::DefNode::Variant(variant) =>
            self::type_of_variant(tcx, variant),
        ir::DefNode::ImplItem(item) => match item.kind {
            ir::ImplItemKind::Fn(..) => tcx.mk_fn_ptr(tcx.fn_sig(def_id)),
        },
        ir::DefNode::ForeignItem(item) => match item.kind {
            ir::ForeignItemKind::Fn(..) => tcx.mk_fn_ptr(tcx.fn_sig(def_id)),
        },
        ir::DefNode::Field(f) => tcx.ir_ty_to_ty(f.ty),
        ir::DefNode::TyParam(_) => panic!(),
    }
}

/// return the type of a function (as a `ty::FnSig`)
fn fn_sig<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId) -> FnSig<'tcx> {
    match tcx.defs().get(def_id) {
        DefNode::Item(item) => match item.kind {
            ir::ItemKind::Fn(sig, ..) => tcx.lower_fn_sig(sig),
            _ => panic!(),
        },
        DefNode::ImplItem(impl_item) => match impl_item.kind {
            ir::ImplItemKind::Fn(sig, ..) => tcx.lower_fn_sig(sig),
        },
        DefNode::ForeignItem(foreign_item) => match foreign_item.kind {
            ir::ForeignItemKind::Fn(sig, ..) => tcx.lower_fn_sig(sig),
        },
        DefNode::Ctor(variant) => {
            let adt = tcx.type_of(variant.adt_def_id);
            match variant.kind {
                ir::VariantKind::Tuple(fields) => FnSig {
                    params: tcx.mk_substs(fields.iter().map(|f| tcx.type_of(f.id.def))),
                    ret: adt,
                },
                _ => panic!("not a constructor function"),
            }
        }
        _ => panic!(),
    }
}

fn type_of_variant<'tcx>(tcx: TyCtx<'tcx>, variant: &'tcx ir::Variant<'tcx>) -> Ty<'tcx> {
    let adt_ty = tcx.type_of(variant.adt_def_id);
    match variant.kind {
        // these two variant kinds are already of the enum type
        ir::VariantKind::Struct(..) | ir::VariantKind::Unit => adt_ty,
        // represent enum tuples as injection functions
        // enum Option<T> {
        //     Some(T),
        //     None
        // }
        //
        // None: Option<T>
        // Some: T -> Option<T>
        ir::VariantKind::Tuple(..) => tcx.mk_fn_ptr(tcx.fn_sig(variant.id.def)),
    }
}

fn type_of_adt<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId) -> Ty<'tcx> {
    let adt = tcx.adt_ty(def_id);
    let substs = Substs::id_for_def(tcx, def_id);
    tcx.mk_adt_ty(adt, substs)
}
