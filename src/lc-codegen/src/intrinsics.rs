use crate::CodegenCtx;
use inkwell::types::BasicType;
use inkwell::values::FunctionValue;
use inkwell::AddressSpace;
use lc_core::ty::Instance;
use lc_span::sym;

impl<'tcx> CodegenCtx<'tcx> {
    pub fn codegen_intrinsic(&self, instance: Instance<'tcx>) {
        if self.intrinsics.borrow().contains_key(&instance) {
            return;
        }
        let ident = self.tcx.defs().ident(instance.def_id);
        let llfn = match ident.symbol {
            sym::addr => self.codegen_addr_intrinsic(instance),
            sym::print => self.native_functions.print,
            _ => panic!("unknown intrinsic `{}`", ident),
        };
        self.intrinsics.borrow_mut().insert(instance, llfn);
    }

    fn codegen_addr_intrinsic(&self, instance: Instance<'tcx>) -> FunctionValue<'tcx> {
        let ident = self.tcx.defs().ident(instance.def_id);
        let name = format!("{}<{}>", ident, instance.substs);
        let t = instance.substs[0];
        let llty = self.llvm_ty(t);
        // `addr<T>: fn(&T) -> int` where the returned int is the address as an i64
        let addr_fn_ty =
            self.types.i64.fn_type(&[llty.ptr_type(AddressSpace::Generic).into()], false);
        let llfn = self.module.add_function(&name, addr_fn_ty, None);
        let block = self.llctx.append_basic_block(llfn, "addr_entry");

        self.position_at_end(block);
        let ptr = llfn.get_first_param().unwrap().into_pointer_value();

        let int = self.build_ptr_to_int(ptr, self.types.i64, "ptr_to_int");
        self.build_return(Some(&int));
        llfn
    }
}
