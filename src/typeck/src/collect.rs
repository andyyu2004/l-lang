use crate::TyConv;
use ir::Visitor;
use lcore::ty::{Substs, Ty, TyCtx};

struct ItemCollector<'tcx> {
    tcx: TyCtx<'tcx>,
}

impl<'tcx> ir::Visitor<'tcx> for ItemCollector<'tcx> {
    fn visit_item(&mut self, item: &ir::Item<'tcx>) {
        let tcx = self.tcx;
        let ty = match &item.kind {
            ir::ItemKind::Fn(sig, generics, _) => {
                let fn_ty = tcx.fn_sig_to_ty(sig);
                tcx.generalize(generics, fn_ty)
            }
            ir::ItemKind::Struct(generics, variant_kind) => {
                let variant_ty = tcx.variant_ty(item.ident, None, variant_kind);
                let adt_ty = tcx.mk_struct_ty(item.id.def, item.ident, variant_ty);
                let substs = Substs::id_for_generics(tcx, tcx.lower_generics(generics));
                let ty = tcx.mk_adt_ty(adt_ty, substs);
                tcx.generalize(generics, ty)
            }
            ir::ItemKind::Enum(generics, variants) => {
                let variant_tys = variants
                    .iter()
                    .map(|variant| {
                        tcx.variant_ty(variant.ident, Some(variant.id.def), &variant.kind)
                    })
                    .collect();

                let adt_ty = tcx.mk_enum_ty(item.id.def, item.ident, variant_tys);
                let substs = Substs::id_for_generics(tcx, tcx.lower_generics(generics));
                let ty = tcx.mk_adt_ty(adt_ty, substs);
                tcx.generalize(generics, ty)
            }
            ir::ItemKind::Impl { generics, trait_path, self_ty, impl_item_refs } => {
                for impl_item_ref in *impl_item_refs {
                    tcx.collect_impl_item(impl_item_ref);
                }
                return;
            }
            ir::ItemKind::Extern(foreign_items) => {
                for foreign_item in *foreign_items {
                    match foreign_item.kind {
                        ir::ForeignItemKind::Fn(sig, generics) => {
                            let fn_ty = tcx.fn_sig_to_ty(sig);
                            let ty = tcx.generalize(generics, fn_ty);
                            tcx.collect_ty(foreign_item.id.def, ty);
                        }
                    }
                }
                return;
            }
        };
        info!("collect item: {:#?}", ty);
        tcx.collect_ty(item.id.def, ty);
    }
}

pub trait TcxCollectExt<'tcx> {
    fn collect_item_types(self);
    fn collect_impl_item(self, impl_item_ref: &ir::ImplItemRef);
    fn generalize(self, generics: &ir::Generics, ty: Ty<'tcx>) -> Ty<'tcx>;
}

impl<'tcx> TcxCollectExt<'tcx> for TyCtx<'tcx> {
    fn collect_item_types(self) {
        collect_item_types(self)
    }

    fn collect_impl_item(self, impl_item_ref: &ir::ImplItemRef) {
        let impl_item = self.impl_item(impl_item_ref.id);
        let ty = match impl_item.kind {
            ir::ImplItemKind::Fn(sig, _) => self.fn_sig_to_ty(sig),
        };
        self.collect_ty(impl_item.id.def, ty);
    }

    fn generalize(self, generics: &ir::Generics, ty: Ty<'tcx>) -> Ty<'tcx> {
        let generics = self.lower_generics(generics);
        self.mk_ty_scheme(generics, ty)
    }
}

pub fn collect_item_types<'tcx>(tcx: TyCtx<'tcx>) {
    ItemCollector { tcx }.visit_prog(tcx.ir);
    CtorCollector { tcx }.visit_prog(tcx.ir);
}

/// this runs a separate collection pass as it requires the enum tys to be known
struct CtorCollector<'tcx> {
    tcx: TyCtx<'tcx>,
}

impl<'tcx> ir::Visitor<'tcx> for CtorCollector<'tcx> {
    fn visit_variant(&mut self, variant: &'tcx ir::Variant<'tcx>) {
        let tcx = self.tcx;
        let ty = tcx.collected_ty(variant.adt_def);
        let (forall, ty) = ty.expect_scheme();
        let (adt_ty, _substs) = ty.expect_adt();
        let ctor_ty = match variant.kind {
            // these two constructor kinds are already of the enum type
            ir::VariantKind::Struct(..) | ir::VariantKind::Unit => ty,
            // represent enum tuples as injection functions
            // enum Option<T> {
            //     Some(T),
            //     None
            // }
            //
            // None: Option<T>
            // Some: T -> Option<T>
            ir::VariantKind::Tuple(..) => {
                let variant = &adt_ty.variants[variant.idx];
                let tys = tcx.mk_substs(variant.fields.iter().map(|f| tcx.ir_ty_to_ty(f.ir_ty)));
                tcx.mk_fn_ty(tys, ty)
            }
        };
        let generalized = tcx.mk_ty_scheme(forall, ctor_ty);
        tcx.collect_ty(variant.id.def, generalized);
    }
}
