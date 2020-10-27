//! contains implementations for collection phases which populate the data structures of the global context
//! this includes type collection, and impl collection

mod inherent;
mod tys;

use crate::TyConv;
use lcore::ty::{Ty, TyCtx};

/// stateful queries that populate the inner data structures of the typing context
pub trait TcxCollectExt<'tcx> {
    fn collect_item_types(self);
    fn collect_inherent_impls(self);
    fn generalize(self, generics: &ir::Generics, ty: Ty<'tcx>) -> Ty<'tcx>;
}

impl<'tcx> TcxCollectExt<'tcx> for TyCtx<'tcx> {
    /// run type collection on items and constructors
    fn collect_item_types(self) {
        tys::collect(self);
    }

    fn generalize(self, generics: &ir::Generics, ty: Ty<'tcx>) -> Ty<'tcx> {
        let generics = self.lower_generics(generics);
        self.mk_ty_scheme(generics, ty)
    }

    fn collect_inherent_impls(self) {
        inherent::collect(self);
    }
}
