use super::TyCtx;
use crate::ast::Ident;
use crate::error::TypeError;
use crate::ir;
use crate::ty::{AdtTy, FieldTy, List, TyConv, TyKind, VariantTy};
use rustc_hash::FxHashMap;

impl<'tcx> TyCtx<'tcx> {
    pub fn collect(self, prog: &ir::Prog<'tcx>) {
        prog.items.values().for_each(|item| self.collect_item(item))
    }

    pub fn collect_item(self, item: &ir::Item<'tcx>) {
        let ty = match item.kind {
            ir::ItemKind::Fn(sig, generics, _body) => {
                let fn_ty = TyConv::fn_sig_to_ty(&self, sig);
                self.generalize(generics, fn_ty)
            }
            ir::ItemKind::Struct(generics, variant_kind) => {
                // TODO
                // let opaque_ty = self.mk_opaque_ty(item.id.def, List::empty());
                // self.item_tys
                //     .borrow_mut()
                //     .insert(item.id.def, self.generalize(generics, opaque_ty));

                let variant_ty = self.variant_ty(item.ident, variant_kind);
                let variant_tys = std::iter::once(variant_ty).collect();
                let adt_ty = self.mk_struct(item.id.def, item.ident, variant_tys);
                let ty = self.mk_empty_adt_ty(adt_ty);
                self.generalize(generics, ty)
            }
        };
        info!("collect: {:#?}", ty);
        self.item_tys.borrow_mut().insert(item.id.def, ty);
    }

    pub fn variant_ty(self, ident: Ident, variant_kind: &ir::VariantKind) -> VariantTy<'tcx> {
        let mut seen = FxHashMap::default();
        let fields = self.arena.alloc_iter(variant_kind.fields().iter().map(|f| {
            if let Some(span) = seen.insert(f.ident, f.span) {
                self.sess.emit_error(span, TypeError::FieldAlreadyDeclared(f.ident, ident));
            }
            FieldTy {
                def_id: f.id.def,
                ident: f.ident,
                vis: f.vis,
                ty: TyConv::ir_ty_to_ty(&self, f.ty),
            }
        }));
        VariantTy { ident, fields }
    }
}
