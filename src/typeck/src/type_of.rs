use crate::TyConv;
use ir::DefId;
use lcore::queries::Queries;
use lcore::ty::*;

pub fn provide(queries: &mut Queries) {
    *queries = Queries { type_of, ..*queries }
}

fn type_of<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId) -> Ty<'tcx> {
    let def_node = tcx.defs().get(def_id);
    match def_node {
        ir::DefNode::Item(item) => match item.kind {
            ir::ItemKind::Fn(sig, ..) =>
                tcx.mk_ty_scheme(tcx.generics_of(def_id), tcx.fn_sig_to_ty(sig)),
            ir::ItemKind::Enum(..) | ir::ItemKind::Struct(..) => self::type_of_adt(tcx, def_id),
            ir::ItemKind::Impl { generics: _, trait_path: _, self_ty, impl_item_refs: _ } =>
                tcx.ir_ty_to_ty(self_ty),
            _ => unreachable!("unexpected item kind in type_of"),
        },
        ir::DefNode::Ctor(variant) | ir::DefNode::Variant(variant) =>
            self::type_of_variant(tcx, variant),
        ir::DefNode::ImplItem(item) => match item.kind {
            ir::ImplItemKind::Fn(sig, ..) =>
                tcx.mk_ty_scheme(tcx.generics_of(def_id), tcx.fn_sig_to_ty(sig)),
        },
        ir::DefNode::ForeignItem(item) => match item.kind {
            ir::ForeignItemKind::Fn(sig, ..) =>
                tcx.mk_ty_scheme(tcx.generics_of(def_id), tcx.fn_sig_to_ty(sig)),
        },
        ir::DefNode::Field(f) => tcx.ir_ty_to_ty(f.ty),
        ir::DefNode::TyParam(_) => panic!(),
    }
}

fn type_of_variant<'tcx>(tcx: TyCtx<'tcx>, variant: &'tcx ir::Variant<'tcx>) -> Ty<'tcx> {
    let ty = tcx.type_of(variant.adt_def_id);
    let (forall, adt_ty) = ty.expect_scheme();
    let (adt, _substs) = adt_ty.expect_adt();
    let ctor_ty = match variant.kind {
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
        ir::VariantKind::Tuple(..) => {
            let variant = &adt.variants[variant.idx];
            let tys = tcx.mk_substs(variant.fields.iter().map(|f| tcx.type_of(f.def_id)));
            tcx.mk_fn_ty(tys, adt_ty)
        }
    };
    tcx.mk_ty_scheme(forall, ctor_ty)
}

fn type_of_adt<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId) -> Ty<'tcx> {
    let adt = tcx.adt_ty(def_id);
    let generics = tcx.generics_of(def_id);
    let substs = Substs::id_for_generics(tcx, generics);
    let ty = tcx.mk_adt_ty(adt, substs);
    tcx.mk_ty_scheme(generics, ty)
}
