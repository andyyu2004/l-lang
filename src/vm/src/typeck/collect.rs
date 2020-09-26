use super::TyCtx;
use crate::ast::Ident;
use crate::error::TypeError;
use crate::ir::{self, DefId, Visitor};
use crate::ty::{AdtTy, FieldTy, List, Ty, TyConv, TyKind, VariantTy};
use ir::CtorKind;
use rustc_hash::FxHashMap;

struct ItemCollector<'tcx> {
    tcx: TyCtx<'tcx>,
}

impl<'tcx> ir::Visitor<'tcx> for ItemCollector<'tcx> {
    fn visit_item(&mut self, item: &ir::Item<'tcx>) {
        let tcx = self.tcx;
        let ty = match &item.kind {
            ir::ItemKind::Fn(sig, generics, _body) => {
                let fn_ty = TyConv::fn_sig_to_ty(&tcx, sig);
                tcx.generalize(generics, fn_ty)
            }
            ir::ItemKind::Struct(generics, variant_kind) => {
                let variant_ty = tcx.variant_ty(item.ident, None, variant_kind);
                let adt_ty = tcx.mk_struct_ty(item.id.def, item.ident, variant_ty);
                let ty = tcx.mk_adt_ty(adt_ty, List::empty());
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
                let ty = tcx.mk_adt_ty(adt_ty, List::empty());
                tcx.generalize(generics, ty)
            }
            ir::ItemKind::Impl { generics, trait_path, self_ty, impl_item_refs } => {
                for impl_item_ref in *impl_item_refs {
                    tcx.collect_impl_item(impl_item_ref);
                }
                return;
            }
        };
        info!("collect item: {:#?}", ty);
        tcx.collect_ty(item.id.def, ty);
    }
}

impl<'tcx> TyCtx<'tcx> {
    pub fn collect(self, prog: &'tcx ir::Prog<'tcx>) {
        ItemCollector { tcx: self }.visit_prog(prog);
        CtorCollector { tcx: self }.visit_prog(prog);
    }

    /// write collected ty to tcx map
    pub fn collect_ty(self, def: DefId, ty: Ty<'tcx>) -> Ty<'tcx> {
        self.collected_tys.borrow_mut().insert(def, ty);
        ty
    }

    pub fn collect_impl_item(self, impl_item_ref: &ir::ImplItemRef) {
        let impl_item = self.impl_item(impl_item_ref.id);
        let ty = match impl_item.kind {
            ir::ImplItemKind::Fn(sig, _) => TyConv::fn_sig_to_ty(&self, sig),
        };
        self.collect_ty(impl_item.id.def, ty);
    }

    pub fn variant_ty(
        self,
        ident: Ident,
        ctor: Option<DefId>,
        variant_kind: &ir::VariantKind<'tcx>,
    ) -> VariantTy<'tcx> {
        let mut seen = FxHashMap::default();
        let fields = self.arena.alloc_iter(variant_kind.fields().iter().map(|f| {
            if let Some(span) = seen.insert(f.ident, f.span) {
                self.sess.emit_error(span, TypeError::FieldAlreadyDeclared(f.ident, ident));
            }
            FieldTy { def_id: f.id.def, ident: f.ident, vis: f.vis, ir_ty: f.ty }
        }));
        VariantTy { ctor, ident, fields, ctor_kind: CtorKind::from(variant_kind) }
    }
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
        let (adt_ty, substs) = ty.expect_adt();
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
                let tys = tcx
                    .mk_substs(variant.fields.iter().map(|f| TyConv::ir_ty_to_ty(&tcx, f.ir_ty)));
                tcx.mk_fn_ty(tys, ty)
            }
        };
        let generalized = tcx.mk_ty_scheme(forall, ctor_ty);
        tcx.collect_ty(variant.id.def, generalized);
    }
}
