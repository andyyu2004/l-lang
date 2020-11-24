use inkwell::context::Context;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::values::*;
use inkwell::{AddressSpace, AtomicOrdering, AtomicRMWBinOp, IntPredicate};
use rustc_hash::FxHashMap;
use std::ops::Deref;

pub struct NativeFunctions<'tcx> {
    pub rc_retain: FxHashMap<(), ()>,
    pub rc_release: FunctionValue<'tcx>,
    pub abort: FunctionValue<'tcx>,
    pub exit: FunctionValue<'tcx>,
    pub print: FunctionValue<'tcx>,
    pub printf: FunctionValue<'tcx>,
    pub print_addr: FunctionValue<'tcx>,
}

pub struct NativeFunctionsBuilder<'a, 'tcx> {
    llcx: &'tcx Context,
    module: &'a Module<'tcx>,
}

impl<'a, 'tcx> NativeFunctionsBuilder<'a, 'tcx> {
    pub fn new(llcx: &'tcx Context, module: &'a Module<'tcx>) -> Self {
        Self { llcx, module }
    }

    pub fn build(&self) -> NativeFunctions<'tcx> {
        let printf = self.build_printf();
        let rc_release = self.build_rc_release();
        let print = self.build_print();
        let print_addr = self.build_print_addr();
        let abort = self.build_abort();
        let exit = self.build_exit();
        NativeFunctions {
            rc_retain: Default::default(),
            rc_release,
            abort,
            print,
            exit,
            print_addr,
            printf,
        }
    }

    fn build_print_addr(&self) -> FunctionValue<'tcx> {
        let unit = self.struct_type(&[], false);
        let printfn = self.module.add_function(
            "print_addr",
            unit.fn_type(&[self.i8_type().ptr_type(AddressSpace::Generic).into()], false),
            None,
        );
        let bb = self.append_basic_block(printfn, "printint");
        let builder = self.create_builder();
        builder.position_at_end(bb);

        let param = printfn.get_first_param().unwrap();
        let vec = self.const_string("%p\n".as_bytes(), true);
        let alloca = builder.build_alloca(vec.get_type(), "alloca_str");
        builder.build_store(alloca, vec);
        let ptr = builder.build_bitcast(
            alloca,
            self.i8_type().ptr_type(AddressSpace::Generic),
            "bitcast",
        );
        let printf = self.module.get_function("printf").unwrap();
        builder.build_call(printf, &[ptr, param], "printf");
        builder.build_return(Some(&self.const_struct(&[], false)));
        printfn
    }

    fn build_print(&self) -> FunctionValue<'tcx> {
        let unit = self.struct_type(&[], false);
        let printfn =
            self.module.add_function("print", unit.fn_type(&[self.i64_type().into()], false), None);
        let bb = self.append_basic_block(printfn, "printint");
        let builder = self.create_builder();
        builder.position_at_end(bb);

        let param = printfn.get_first_param().unwrap();
        let vec = self.const_string("%d\n".as_bytes(), true);
        let alloca = builder.build_alloca(self.i8_type().array_type(4), "alloca_str");
        builder.build_store(alloca, vec);
        let ptr = builder.build_bitcast(
            alloca,
            self.i8_type().ptr_type(AddressSpace::Generic),
            "bitcast",
        );
        let printf = self.module.get_function("printf").unwrap();
        builder.build_call(printf, &[ptr, param], "printf");
        builder.build_return(Some(&self.const_struct(&[], false)));
        printfn
    }

    fn build_printf(&self) -> FunctionValue<'tcx> {
        self.module.add_function(
            "printf",
            self.i32_type().fn_type(&[self.i8_type().ptr_type(AddressSpace::Generic).into()], true),
            Some(Linkage::External),
        )
    }

    fn build_exit(&self) -> FunctionValue<'tcx> {
        self.module.add_function(
            "exit",
            self.void_type().fn_type(&[self.i32_type().into()], false),
            Some(Linkage::External),
        )
    }

    fn build_abort(&self) -> FunctionValue<'tcx> {
        self.module.add_function(
            "abort",
            self.void_type().fn_type(&[], false),
            Some(Linkage::External),
        )
    }

    fn build_rc_release(&self) -> FunctionValue<'tcx> {
        let rc_release = self.module.add_function(
            "rc_release",
            self.void_type().fn_type(
                &[
                    self.i8_type().ptr_type(AddressSpace::Generic).into(),
                    self.i32_type().ptr_type(AddressSpace::Generic).into(),
                ],
                false,
            ),
            None,
        );
        // build the function
        let builder = self.create_builder();
        let block = self.append_basic_block(rc_release, "rc_release");
        // this is the pointer to be freed
        let ptr = rc_release.get_first_param().unwrap().into_pointer_value();
        // this should be a pointer to the refcount itself
        let rc_ptr = rc_release.get_nth_param(1).unwrap().into_pointer_value();
        builder.position_at_end(block);
        // the refcount is an i32 partially because i64 is too large and it helps a lot with
        // catching type errors as to not confuse it with the i64 type used in l itself
        let one = self.i32_type().const_int(1, false);
        let ref_count = builder
            .build_atomicrmw(
                AtomicRMWBinOp::Sub,
                rc_ptr,
                one,
                AtomicOrdering::SequentiallyConsistent,
            )
            .unwrap();
        let then = self.append_basic_block(rc_release, "free");
        let els = self.append_basic_block(rc_release, "ret");
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

impl<'a, 'tcx> Deref for NativeFunctionsBuilder<'a, 'tcx> {
    // this needs to be a 'tcx lifetime
    // so it returns a & &'tcx Contenxt
    // this is fine as autoderef can deal with this
    type Target = &'tcx Context;

    fn deref(&self) -> &Self::Target {
        &self.llcx
    }
}
