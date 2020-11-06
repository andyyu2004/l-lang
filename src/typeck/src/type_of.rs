use crate::TyConv;
use ir::DefId;
use lcore::queries::Queries;
use lcore::ty::*;

pub fn provide(queries: &mut Queries) {
    *queries = Queries { type_of, ..*queries }
}

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

fn type_of<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId) -> Ty<'tcx> {
    let def_node = tcx.defs().get(def_id);
    match def_node {
        ir::DefNode::Item(item) => match item.kind {
            ir::ItemKind::Fn(sig, ..) => tcx.fn_sig_to_ty(sig),
            ir::ItemKind::Enum(..) | ir::ItemKind::Struct(..) => type_of_adt(tcx, def_id),
            ir::ItemKind::Impl { generics: _, trait_path: _, self_ty, impl_item_refs: _ } =>
                tcx.ir_ty_to_ty(self_ty),
            _ => unreachable!("unexpected item kind in type_of"),
        },
        ir::DefNode::Ctor(variant) | ir::DefNode::Variant(variant) => type_of_variant(tcx, variant),
        ir::DefNode::ImplItem(item) => match item.kind {
            ir::ImplItemKind::Fn(sig, ..) => tcx.fn_sig_to_ty(sig),
        },
        ir::DefNode::ForeignItem(item) => match item.kind {
            ir::ForeignItemKind::Fn(sig, ..) => tcx.fn_sig_to_ty(sig),
        },
        ir::DefNode::TyParam(_) => panic!(),
    }
}

fn type_of_variant<'tcx>(tcx: TyCtx<'tcx>, variant: ir::Variant<'tcx>) -> Ty<'tcx> {
    let ty = tcx.type_of(variant.adt_def_id);
    let (forall, adt_ty) = ty.expect_scheme();
    let (adt, _substs) = adt_ty.expect_adt();
    let ctor_ty = match variant.kind {
        // these two constructor kinds are already of the enum type
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
            let tys = tcx.mk_substs(variant.fields.iter().map(|f| tcx.ir_ty_to_ty(f.ir_ty)));
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
