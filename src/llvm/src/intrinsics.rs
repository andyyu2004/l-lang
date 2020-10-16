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
            sym::RC => self.codegen_rc_intrinsic(instance),
            sym::PRINT => self.codegen_print_intrinsic(instance),
            _ => panic!("unknown intrinsic `{}`", ident),
        };
        self.intrinsics.borrow_mut().insert(instance, llfn);
    }

    fn codegen_print_intrinsic(&self, _instance: Instance<'tcx>) -> FunctionValue<'tcx> {
        let printfn = self.module.add_function(
            "print",
            self.types.unit.fn_type(&[self.types.int.into()], false),
            None,
        );
        let bb = self.llctx.append_basic_block(printfn, "printint");
        self.position_at_end(bb);

        let param = printfn.get_first_param().unwrap();
        let vec = self.llctx.const_string("%d\n".as_bytes(), true);
        let alloca = self.build_alloca(self.llctx.i8_type().array_type(4), "alloca_str");
        self.build_store(alloca, vec);
        let ptr =
            self.build_bitcast(alloca, self.types.byte.ptr_type(AddressSpace::Generic), "bitcast");
        self.build_call(self.native_functions.printf, &[ptr, param], "printf");
        self.build_return(Some(&self.vals.unit));
        printfn
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
