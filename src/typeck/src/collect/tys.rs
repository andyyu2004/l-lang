//! collect item types

use crate::TyConv;
use index::Idx;
use ir::{CtorKind, DefId, VariantIdx, Visitor};
use lcore::queries::Queries;
use lcore::ty::{self, AdtKind, AdtTy, FieldTy, Substs, TyCtx, TypeError, VariantTy};
use rustc_hash::FxHashMap;

crate fn provide(queries: &mut Queries) {
    *queries = Queries { adt_ty, ..*queries }
}

fn adt_ty<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId) -> &'tcx AdtTy<'tcx> {
    let item = match tcx.defs().get(def_id) {
        ir::DefNode::Item(item) => item,
        _ => panic!(),
    };

    let (kind, variants) = match item.kind {
        ir::ItemKind::Enum(_, variants) => {
            let variants = variants.iter().map(|variant| variant_ty(tcx, variant)).collect();
            (AdtKind::Enum, variants)
        }
        ir::ItemKind::Struct(_, kind) => {
            // little bit hacky, turning the variant kind into a variant
            let ir::Item { id, span, ident, .. } = item;
            let variant =
                ir::Variant { id, ident, span, adt_def_id: id.def, kind, idx: VariantIdx::new(0) };
            let variant = std::iter::once(&variant).map(|v| variant_ty(tcx, v)).collect();

            (AdtKind::Struct, variant)
        }
        _ => panic!(),
    };

    let adt = tcx.mk_adt(def_id, kind, item.ident, variants);

    // enforces that the fields of the adt are ok
    // even if they are never accessed
    for variant in &adt.variants {
        for field in variant.fields {
            tcx.ir_ty_to_ty(field.ir_ty);
        }
    }

    adt
}

fn variant_ty<'tcx>(tcx: TyCtx<'tcx>, variant: &ir::Variant<'tcx>) -> VariantTy<'tcx> {
    let &ir::Variant { id, ident, kind, .. } = variant;

    let mut seen = FxHashMap::default();
    let fields = tcx.alloc_iter(kind.fields().iter().map(|f| {
        if let Some(span) = seen.insert(f.ident, f.span) {
            tcx.sess
                .emit_error(vec![f.span, span], TypeError::FieldAlreadyDeclared(f.ident, ident));
        }
        FieldTy { def_id: f.id.def, ident: f.ident, vis: f.vis, ir_ty: f.ty }
    }));

    // assign the def_id of the variant as the constructor id
    let ctor = match kind {
        ir::VariantKind::Tuple(..) => Some(id.def),
        ir::VariantKind::Struct(..) | ir::VariantKind::Unit => None,
    };

    VariantTy { ctor, ident, fields, ctor_kind: CtorKind::from(&kind) }
}

// crate fn collect<'tcx>(tcx: TyCtx<'tcx>) {
//     // we must do this in multiple phases as functions
//     // may need to refer to adt defintions
//     AdtCollector::new(tcx).visit_ir(tcx.ir);
//     FnCollector { tcx }.visit_ir(tcx.ir);
// }

// struct AdtCollector<'tcx> {
//     tcx: TyCtx<'tcx>,
//     /// holds a list of adt tys that needs to have their fields checked after collection
//     /// this because the fields are never lowered into ty::Ty representations and only as ir::Ty
//     /// due to the potentially recursive nature of adts
//     /// some validation is performed during `ir_ty_to_ty` and so we run every field in every
//     /// variant of every adt through `ir_ty_to_ty` to check for errors that would otherwise be
//     /// uncaught
//     adts: Vec<&'tcx AdtTy<'tcx>>,
// }

// impl<'tcx> AdtCollector<'tcx> {
//     fn new(tcx: TyCtx<'tcx>) -> Self {
//         Self { tcx, adts: Default::default() }
//     }

//     fn check_adt_variants(&self, adt: &AdtTy<'tcx>) {
//         for variant in &adt.variants {
//             for field in variant.fields {
//                 self.tcx.ir_ty_to_ty(field.ir_ty);
//             }
//         }
//     }
// }

// impl<'tcx> ir::Visitor<'tcx> for AdtCollector<'tcx> {
//     fn visit_ir(&mut self, ir: &'tcx ir::Ir<'tcx>) {
//         ir::walk_ir(self, ir);
//         // we run the checks after all adts are collected as they may reference one another
//         for &adt in &self.adts {
//             self.check_adt_variants(adt)
//         }
//     }

//     fn visit_item(&mut self, item: &ir::Item<'tcx>) {
//         let tcx = self.tcx;
//         match &item.kind {
//             ir::ItemKind::Struct(generics, variant_kind) => {
//                 let variant_ty = variant_ty(tcx, item.ident, None, variant_kind);
//                 let adt_ty = tcx.mk_struct_ty(item.id.def, item.ident, variant_ty);
//                 self.adts.push(adt_ty);
//                 let generics = tcx.lower_generics(generics);
//                 let substs = Substs::id_for_generics(tcx, generics);
//                 let ty = tcx.mk_adt_ty(adt_ty, substs);
//                 tcx.collect_ty(item.id.def, tcx.mk_ty_scheme(generics, ty));
//             }
//             ir::ItemKind::Enum(generics, variants) => {
//                 let variant_tys = variants
//                     .iter()
//                     .map(|variant| {
//                         variant_ty(tcx, variant.ident, Some(variant.id.def), &variant.kind)
//                     })
//                     .collect();

//                 let adt_ty = tcx.mk_enum_ty(item.id.def, item.ident, variant_tys);
//                 self.adts.push(adt_ty);
//                 let generics = tcx.lower_generics(generics);
//                 let substs = Substs::id_for_generics(tcx, generics);
//                 let ty = tcx.mk_adt_ty(adt_ty, substs);
//                 tcx.collect_ty(item.id.def, tcx.mk_ty_scheme(generics, ty));
//                 self.check_adt_variants(adt_ty);
//             }
//             _ => {}
//         };
//     }
// }

// struct FnCollector<'tcx> {
//     tcx: TyCtx<'tcx>,
// }

// impl<'tcx> ir::Visitor<'tcx> for FnCollector<'tcx> {
//     fn visit_item(&mut self, item: &'tcx ir::Item<'tcx>) {
//         let tcx = self.tcx;
//         match &item.kind {
//             ir::ItemKind::Fn(sig, generics, _) => {
//                 let fn_ty = tcx.fn_sig_to_ty(sig);
//                 let generics = tcx.lower_generics(generics);
//                 let ty = tcx.mk_ty_scheme(generics, fn_ty);
//                 tcx.collect_ty(item.id.def, ty);
//             }
//             ir::ItemKind::Impl { generics, trait_path: _, self_ty: _, impl_item_refs } => {
//                 let impl_generics = tcx.lower_generics(generics);
//                 for impl_item_ref in *impl_item_refs {
//                     collect_impl_item(tcx, impl_generics, impl_item_ref);
//                 }
//                 return;
//             }
//             ir::ItemKind::Extern(foreign_items) => {
//                 for foreign_item in *foreign_items {
//                     match foreign_item.kind {
//                         ir::ForeignItemKind::Fn(sig, generics) => {
//                             let fn_ty = tcx.fn_sig_to_ty(sig);
//                             let generics = tcx.lower_generics(generics);
//                             let ty = tcx.mk_ty_scheme(generics, fn_ty);
//                             tcx.collect_ty(foreign_item.id.def, ty);
//                         }
//                     }
//                 }
//                 return;
//             }
//             ir::ItemKind::Enum(..) => ir::walk_item(self, item),
//             ir::ItemKind::Struct(..) => {}
//         };
//     }

//     fn visit_variant(&mut self, variant: &'tcx ir::Variant<'tcx>) {
//         let tcx = self.tcx;
//         let ty = tcx.collected_ty(variant.adt_def_id);
//         let (forall, ty) = ty.expect_scheme();
//         let (adt_ty, _substs) = ty.expect_adt();
//         let ctor_ty = match variant.kind {
//             // these two constructor kinds are already of the enum type
//             ir::VariantKind::Struct(..) | ir::VariantKind::Unit => ty,
//             // represent enum tuples as injection functions
//             // enum Option<T> {
//             //     Some(T),
//             //     None
//             // }
//             //
//             // None: Option<T>
//             // Some: T -> Option<T>
//             ir::VariantKind::Tuple(..) => {
//                 let variant = &adt_ty.variants[variant.idx];
//                 let tys = tcx.mk_substs(variant.fields.iter().map(|f| tcx.ir_ty_to_ty(f.ir_ty)));
//                 tcx.mk_fn_ty(tys, ty)
//             }
//         };
//         let generalized = tcx.mk_ty_scheme(forall, ctor_ty);
//         tcx.collect_ty(variant.id.def, generalized);
//     }
// }

// fn collect_impl_item<'tcx>(
//     tcx: TyCtx<'tcx>,
//     impl_generics: &'tcx ty::Generics<'tcx>,
//     impl_item_ref: &ir::ImplItemRef,
// ) {
//     let impl_item = tcx.impl_item(impl_item_ref.id);
//     let ty = match impl_item.kind {
//         ir::ImplItemKind::Fn(sig, _) => tcx.fn_sig_to_ty(sig),
//     };
//     let item_generics = tcx.lower_generics(impl_item.generics);
//     let generics = tcx.concat_generics(impl_generics, item_generics);
//     let generalized = tcx.mk_ty_scheme(generics, ty);
//     tcx.collect_ty(impl_item.id.def, generalized);
// }
