use crate::CodegenCtx;
use inkwell::types::BasicType;
use inkwell::values::FunctionValue;
use inkwell::AddressSpace;
use lcore::ty::Instance;
use span::sym;

impl<'tcx> CodegenCtx<'tcx> {
    pub fn codegen_intrinsic(&self, instance: Instance<'tcx>) {
        let ident = self.tcx.defs().ident_of(instance.def_id);
        let llfn = match ident.symbol {
            sym::RC => self.codegen_rc_intrinsic(instance),
            _ => panic!(),
        };
        self.intrinsics.borrow_mut().insert(instance, llfn);
    }

    fn codegen_rc_intrinsic(&self, instance: Instance<'tcx>) -> FunctionValue<'tcx> {
        let ident = self.tcx.defs().ident_of(instance.def_id);
        let name = format!("{}<{}>", ident, instance.substs);
        // the generic parameter of the `rc` intrinsic
        let t = instance.substs[0];
        let llty = self.llvm_ty(t);
        // `rc<T>: &T -> int`
        let rc_fn_ty =
            self.types.int.fn_type(&[llty.ptr_type(AddressSpace::Generic).into()], false);
        let llfn = self.module.add_function(&name, rc_fn_ty, None);
        let block = self.llctx.append_basic_block(llfn, "rc_entry");

        self.position_at_end(block);
        let ptr = llfn.get_first_param().unwrap().into_pointer_value();
        let cast = self.build_pointer_cast(
            ptr,
            self.llctx
                .struct_type(&[self.types.int.into(), llty], false)
                .ptr_type(AddressSpace::Generic),
            "cast_box_ptr",
        );
        let refcount_ptr = self.build_struct_gep(cast, 1, "rc_gep").unwrap();
        let refcount = self.build_load(refcount_ptr, "load_refcount").into_int_value();
        let i64_rc = self.build_int_cast(refcount, self.types.int, "rc->i64");
        self.build_return(Some(&i64_rc));
        llfn
    }
}
