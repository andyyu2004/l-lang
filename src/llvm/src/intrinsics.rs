use crate::CodegenCtx;
use inkwell::types::BasicType;
use inkwell::values::FunctionValue;
use inkwell::AddressSpace;
use lcore::ty::Instance;
use span::sym;

impl<'tcx> CodegenCtx<'tcx> {
    pub fn codegen_intrinsic(&self, instance: Instance<'tcx>) {
        if self.intrinsics.borrow().contains_key(&instance) {
            return;
        }
        let ident = self.tcx.defs().ident_of(instance.def_id);
        let llfn = match ident.symbol {
            sym::rc => self.codegen_rc_intrinsic(instance),
            sym::addr => self.codegen_addr_intrinsic(instance),
            sym::print => self.native_functions.print,
            _ => panic!("unknown intrinsic `{}`", ident),
        };
        self.intrinsics.borrow_mut().insert(instance, llfn);
    }

    fn codegen_addr_intrinsic(&self, instance: Instance<'tcx>) -> FunctionValue<'tcx> {
        let ident = self.tcx.defs().ident_of(instance.def_id);
        let name = format!("{}<{}>", ident, instance.substs);
        let t = instance.substs[0];
        let llty = self.llvm_ty(t);
        // `addr<T>: fn(&T) -> int` where the returned int is the address as a u64
        let addr_fn_ty =
            self.types.int.fn_type(&[llty.ptr_type(AddressSpace::Generic).into()], false);
        let llfn = self.module.add_function(&name, addr_fn_ty, None);
        let block = self.llctx.append_basic_block(llfn, "addr_entry");

        self.position_at_end(block);
        let ptr = llfn.get_first_param().unwrap().into_pointer_value();

        let int = self.build_ptr_to_int(ptr, self.types.int, "ptr_to_int");
        self.build_return(Some(&int));
        llfn
    }

    fn codegen_rc_intrinsic(&self, instance: Instance<'tcx>) -> FunctionValue<'tcx> {
        let ident = self.tcx.defs().ident_of(instance.def_id);
        let name = format!("{}<{}>", ident, instance.substs);
        // `t` is the generic parameter of the `rc` intrinsic
        let t = instance.substs[0];
        let llty = self.llvm_ty(t);
        // `rc<T>: fn(&T) -> int`
        let rc_fn_ty =
            self.types.int.fn_type(&[llty.ptr_type(AddressSpace::Generic).into()], false);
        let llfn = self.module.add_function(&name, rc_fn_ty, None);
        let block = self.llctx.append_basic_block(llfn, "rc_entry");

        self.position_at_end(block);
        let ptr = llfn.get_first_param().unwrap().into_pointer_value();
        let cast = self.build_pointer_cast(
            ptr,
            self.llctx
                .struct_type(&[llty, self.types.discr.into()], false)
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
