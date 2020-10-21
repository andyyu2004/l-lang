use inkwell::context::Context;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::values::*;
use inkwell::{AddressSpace, AtomicOrdering, AtomicRMWBinOp, IntPredicate};
use rustc_hash::FxHashMap;

pub struct NativeFunctions<'tcx> {
    pub rc_retain: FxHashMap<(), ()>,
    pub rc_release: FunctionValue<'tcx>,
    // pub print_int: FunctionValue<'tcx>,
    pub abort: FunctionValue<'tcx>,
    pub exit: FunctionValue<'tcx>,
    pub print: FunctionValue<'tcx>,
    pub printf: FunctionValue<'tcx>,
    pub print_addr: FunctionValue<'tcx>,
}

impl<'tcx> NativeFunctions<'tcx> {
    pub fn new(module: &Module<'tcx>) -> Self {
        let rc_release = Self::build_rc_release(module);
        let printf = Self::build_printf(module);
        let print = Self::build_print(module);
        let print_addr = Self::build_print_addr(module);
        let abort = Self::build_abort(module);
        let exit = Self::build_exit(module);
        Self { rc_retain: Default::default(), rc_release, abort, print, exit, print_addr, printf }
    }

    fn build_print_addr(module: &Module<'tcx>) -> FunctionValue<'tcx> {
        let llctx = module.get_context().get();
        let unit = llctx.struct_type(&[], false);
        let printfn = module.add_function(
            "print_addr",
            unit.fn_type(&[llctx.i8_type().ptr_type(AddressSpace::Generic).into()], false),
            None,
        );
        let bb = llctx.append_basic_block(printfn, "printint");
        let builder = llctx.create_builder();
        builder.position_at_end(bb);

        let param = printfn.get_first_param().unwrap();
        let vec = llctx.const_string("%p\n".as_bytes(), true);
        let alloca = builder.build_alloca(vec.get_type(), "alloca_str");
        builder.build_store(alloca, vec);
        let ptr = builder.build_bitcast(
            alloca,
            llctx.i8_type().ptr_type(AddressSpace::Generic),
            "bitcast",
        );
        let printf = module.get_function("printf").unwrap();
        builder.build_call(printf, &[ptr, param], "printf");
        builder.build_return(Some(&llctx.const_struct(&[], false)));
        printfn
    }

    fn build_print(module: &Module<'tcx>) -> FunctionValue<'tcx> {
        let llctx = module.get_context().get();
        let unit = llctx.struct_type(&[], false);
        let printfn =
            module.add_function("print", unit.fn_type(&[llctx.i64_type().into()], false), None);
        let bb = llctx.append_basic_block(printfn, "printint");
        let builder = llctx.create_builder();
        builder.position_at_end(bb);

        let param = printfn.get_first_param().unwrap();
        let vec = llctx.const_string("%d\n".as_bytes(), true);
        let alloca = builder.build_alloca(llctx.i8_type().array_type(4), "alloca_str");
        builder.build_store(alloca, vec);
        let ptr = builder.build_bitcast(
            alloca,
            llctx.i8_type().ptr_type(AddressSpace::Generic),
            "bitcast",
        );
        let printf = module.get_function("printf").unwrap();
        builder.build_call(printf, &[ptr, param], "printf");
        builder.build_return(Some(&llctx.const_struct(&[], false)));
        printfn
    }

    fn build_printf(module: &Module<'tcx>) -> FunctionValue<'tcx> {
        let llctx = module.get_context().get();
        module.add_function(
            "printf",
            llctx
                .i32_type()
                .fn_type(&[llctx.i8_type().ptr_type(AddressSpace::Generic).into()], true),
            Some(Linkage::External),
        )
    }

    fn build_exit(module: &Module<'tcx>) -> FunctionValue<'tcx> {
        let llctx = module.get_context().get();
        module.add_function(
            "exit",
            llctx.void_type().fn_type(&[llctx.i32_type().into()], false),
            Some(Linkage::External),
        )
    }

    fn build_abort(module: &Module<'tcx>) -> FunctionValue<'tcx> {
        let llctx = module.get_context().get();
        module.add_function("abort", llctx.void_type().fn_type(&[], false), Some(Linkage::External))
    }

    fn build_rc_retain(llctx: &'tcx Context, module: &Module<'tcx>) -> FunctionValue<'tcx> {
        let rc_retain = module.add_function(
            "rc_retain",
            llctx
                .void_type()
                .fn_type(&[llctx.i8_type().ptr_type(AddressSpace::Generic).into()], false),
            None,
        );

        let ptr = rc_retain.get_first_param().unwrap().into_pointer_value();

        rc_retain
    }

    fn build_rc_release(module: &Module<'tcx>) -> FunctionValue<'tcx> {
        let llctx = module.get_context().get();
        let rc_release = module.add_function(
            "rc_release",
            llctx.void_type().fn_type(
                &[
                    llctx.i8_type().ptr_type(AddressSpace::Generic).into(),
                    llctx.i32_type().ptr_type(AddressSpace::Generic).into(),
                ],
                false,
            ),
            None,
        );
        // build the function
        let builder = llctx.create_builder();
        let block = llctx.append_basic_block(rc_release, "rc_release");
        // this is the pointer to be freed
        let ptr = rc_release.get_first_param().unwrap().into_pointer_value();
        // this should be a pointer to the refcount itself
        let rc_ptr = rc_release.get_nth_param(1).unwrap().into_pointer_value();
        builder.position_at_end(block);
        // the refcount is an i32 partially because i64 is too large and it helps a lot with
        // catching type errors as to not confuse it with the i64 type used in l itself
        let one = llctx.i32_type().const_int(1, false);
        let ref_count = builder
            .build_atomicrmw(
                AtomicRMWBinOp::Sub,
                rc_ptr,
                one,
                AtomicOrdering::SequentiallyConsistent,
            )
            .unwrap();
        let then = llctx.append_basic_block(rc_release, "free");
        let els = llctx.append_basic_block(rc_release, "ret");
        // this ref_count is the count before decrement if refcount == 1 then this the last
        // reference and we can free it using less than comparison rather than just equality as
        // this will result in certain refcount errors to result in double frees and hopefully
        // crash our program
        let cmp = builder.build_int_compare(IntPredicate::ULE, ref_count, one, "rc_cmp");
        builder.build_conditional_branch(cmp, then, els);
        // build trivial else branch
        builder.position_at_end(els);
        builder.build_return(None);

        // build code to free the ptr
        builder.position_at_end(then);
        // conveniently, the pointer passed to free does not need to be the
        // same type as the one given during the malloc call (I think)
        // if it's anything like C, then malloc takes a void pointer
        // but it must be the same address
        builder.build_free(ptr);
        builder.build_return(None);
        rc_release
    }
}
