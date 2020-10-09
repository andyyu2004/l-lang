use super::CodegenCtx;
use ast::Ident;
use ir::{self, DefId, FnVisitor};
use lcore::ty::{Instance, Subst, Substs};
use span::sym;
use std::ops::Deref;

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
        // special case for the main function
        // as main is not necessarily called, we don't generate any monomorphizations for it
        if Some(def_id) == self.tcx.ir.entry_id {
            assert_eq!(ty, self.tcx.types.main);
            let llfn =
                self.module.add_function(sym::MAIN.as_str(), self.llvm_fn_ty_from_ty(ty), None);
            self.cctx.instances.borrow_mut().insert(Instance::item(Substs::empty(), def_id), llfn);
            return;
        }

        let monomorphizations = match self.tcx.monomorphizations_for(def_id) {
            Some(mono) => mono,
            // if there are no monomorphizations recorded, then presumably it is never called
            None => return,
        };

        // define a new function for every monomorphization
        for substs in monomorphizations {
            dbg!(substs);
            let llty = self.llvm_fn_ty_from_ty(ty.subst(self.tcx, substs));
            let name = format!("{}<{}>", ident, substs);
            let llfn = self.module.add_function(&name, llty, None);
            self.cctx.instances.borrow_mut().insert(Instance::item(substs, def_id), llfn);
        }
    }

    // fn visit_foreign_fn(
    //     &mut self,
    //     def_id: DefId,
    //     ident: Ident,
    //     _sig: &'tcx ir::FnSig<'tcx>,
    //     _generics: &'tcx ir::Generics<'tcx>,
    // ) {
    //     // assume that all foreign functions are intrinsics for now
    //     self.cctx.instances.borrow_mut().insert(def_id, self.intrinsics[&ident.symbol]);
    // }

    // this function actually declares AND codegens
    // the enum constructors as its much more convenient and
    // they don't have the issue of referring to other functions
    // fn visit_enum(&mut self, item: &ir::Item) {
    //     for (ctor_id, (ident, mir)) in mir::build_enum_ctors(self.tcx, item) {
    //         let (_, ty) = self.tcx.collected_ty(ctor_id).expect_scheme();
    //         let llty = self.llvm_fn_ty_from_ty(ty);
    //         let llfn = self.module.add_function(ident.as_str(), llty, None);
    //         self.cctx.instances.borrow_mut().insert(ctor_id, llfn);
    //         FnCtx::new(self, llfn, mir).codegen();
    //     }
    // }
}

pub struct MirCollector<'a, 'tcx> {
    pub cctx: &'a CodegenCtx<'tcx>,
}

impl<'tcx> FnVisitor<'tcx> for MirCollector<'_, 'tcx> {
    fn visit_fn(
        &mut self,
        def_id: DefId,
        _ident: Ident,
        sig: &'tcx ir::FnSig<'tcx>,
        generics: &'tcx ir::Generics<'tcx>,
        body: &'tcx ir::Body<'tcx>,
    ) {
        if let Ok(mir) = mir::build_mir(self.tcx, def_id, sig, generics, body) {
            eprintln!("{}", mir);
            self.mir_bodies.borrow_mut().insert(def_id, mir);
        }
    }
}

impl<'a, 'tcx> Deref for MirCollector<'a, 'tcx> {
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
