use inkwell::context::ContextRef;
use inkwell::module::Module;
use inkwell::values::FunctionValue;
use inkwell::AddressSpace;
use rustc_hash::FxHashMap;
use span::{sym, Symbol};

trait InstrinsicsExt<'tcx> {
    fn llctx(&self) -> ContextRef<'tcx>;
    fn build_rc(&self) -> FunctionValue<'tcx>;
}

impl<'tcx> InstrinsicsExt<'tcx> for Module<'tcx> {
    fn llctx(&self) -> ContextRef<'tcx> {
        self.get_context()
    }

    fn build_rc(&self) -> FunctionValue<'tcx> {
        let llctx = self.llctx().get();
        let rc = self.add_function(
            "rc",
            llctx
                .i64_type()
                .fn_type(&[llctx.i64_type().ptr_type(AddressSpace::Generic).into()], false),
            None,
        );
        // only works for &int currently
        let builder = llctx.create_builder();
        let block = llctx.append_basic_block(rc, "rc_entry");
        builder.position_at_end(block);

        let ptr = rc.get_first_param().unwrap().into_pointer_value();
        let cast = builder.build_pointer_cast(
            ptr,
            llctx
                .struct_type(&[llctx.i64_type().into(), llctx.i32_type().into()], false)
                .ptr_type(AddressSpace::Generic),
            "sdf",
        );
        let refcount_ptr = builder.build_struct_gep(cast, 1, "rc_gep").unwrap();
        let refcount = builder.build_load(refcount_ptr, "load_refcount").into_int_value();
        let i64_rc = builder.build_int_cast(refcount, llctx.i64_type(), "rc->i64");
        builder.build_return(Some(&i64_rc));
        rc
    }
}

pub fn build_instrinsics<'tcx>(module: &Module<'tcx>) -> FxHashMap<Symbol, FunctionValue<'tcx>> {
    std::array::IntoIter::new([(sym::RC, module.build_rc())]).collect()
}
