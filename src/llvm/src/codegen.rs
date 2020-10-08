use super::{CodegenCtx, FnCtx};
use ast::Ident;
use ir::{self, DefId, FnVisitor};
use std::ops::Deref;

/// runs first pass of codegen where all functions are declared
pub struct DeclarationCollector<'a, 'tcx> {
    pub cctx: &'a CodegenCtx<'tcx>,
}

impl<'tcx> FnVisitor<'tcx> for DeclarationCollector<'_, 'tcx> {
    fn visit_fn(
        &mut self,
        def_id: DefId,
        ident: Ident,
        _sig: &'tcx ir::FnSig<'tcx>,
        _generics: &'tcx ir::Generics<'tcx>,
        _body: &'tcx ir::Body<'tcx>,
    ) {
        let (_, ty) = self.tcx.collected_ty(def_id).expect_scheme();
        let (params, ret) = ty.expect_fn();
        let llty = self.llvm_fn_ty(params, ret);
        let llfn = self.module.add_function(ident.as_str(), llty, None);
        self.cctx.items.borrow_mut().insert(def_id, llfn);
    }

    fn visit_foreign_fn(
        &mut self,
        def_id: DefId,
        ident: Ident,
        _sig: &'tcx ir::FnSig<'tcx>,
        _generics: &'tcx ir::Generics<'tcx>,
    ) {
        // assume that all foreign functions are intrinsics for now
        self.cctx.items.borrow_mut().insert(def_id, self.intrinsics[&ident.symbol]);
    }

    // this function actually declares AND codegens
    // the enum constructors as its much more convenient and
    // they don't have the issue of referring to other functions
    fn visit_enum(&mut self, item: &ir::Item) {
        for (ctor_id, (ident, mir)) in mir::build_enum_ctors(self.tcx, item) {
            let (_, ty) = self.tcx.collected_ty(ctor_id).expect_scheme();
            let llty = self.llvm_fn_ty_from_ty(ty);
            let llfn = self.module.add_function(ident.as_str(), llty, None);
            self.cctx.items.borrow_mut().insert(ctor_id, llfn);
            FnCtx::new(self, llfn, mir).codegen();
        }
    }
}

pub struct CodegenCollector<'a, 'tcx> {
    pub cctx: &'a CodegenCtx<'tcx>,
}

impl<'tcx> FnVisitor<'tcx> for CodegenCollector<'_, 'tcx> {
    fn visit_fn(
        &mut self,
        def_id: DefId,
        ident: Ident,
        sig: &'tcx ir::FnSig<'tcx>,
        generics: &'tcx ir::Generics<'tcx>,
        body: &'tcx ir::Body<'tcx>,
    ) {
        let llfn = self.module.get_function(ident.as_str()).unwrap();
        if let Ok(mir) = mir::build_mir(self.tcx, def_id, sig, generics, body) {
            eprintln!("{}", mir);
            FnCtx::new(self, llfn, mir).codegen();
        }
    }
}

impl<'a, 'tcx> Deref for CodegenCollector<'a, 'tcx> {
    type Target = CodegenCtx<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.cctx
    }
}

impl<'a, 'tcx> Deref for DeclarationCollector<'a, 'tcx> {
    type Target = CodegenCtx<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.cctx
    }
}
