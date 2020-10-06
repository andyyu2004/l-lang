use inkwell::context::Context;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::values::*;
use inkwell::{AddressSpace, AtomicOrdering, AtomicRMWBinOp, IntPredicate};
use rustc_hash::FxHashMap;

pub struct NativeFunctions<'tcx> {
    pub rc_retain: FxHashMap<(), ()>,
    pub rc_release: FunctionValue<'tcx>,
    pub print_int: FunctionValue<'tcx>,
    pub abort: FunctionValue<'tcx>,
    pub exit: FunctionValue<'tcx>,
    pub printf: FunctionValue<'tcx>,
}

#[no_mangle]
pub extern "C" fn iprintln(i: i32) {
    println!("{}", i);
}

impl<'tcx> NativeFunctions<'tcx> {
    pub fn new(llctx: &'tcx Context, module: &Module<'tcx>) -> Self {
        let rc_release = Self::build_rc_release(llctx, module);
        let iprintln = Self::build_iprintln(llctx, module);
        let printf = Self::build_printf(llctx, module);
        let abort = Self::build_abort(llctx, module);
        let exit = Self::build_exit(llctx, module);
        Self { rc_retain: Default::default(), rc_release, print_int: iprintln, abort, printf, exit }
    }

    fn build_printf(llctx: &'tcx Context, module: &Module<'tcx>) -> FunctionValue<'tcx> {
        module.add_function("printf", llctx.i32_type().fn_type(&[], false), Some(Linkage::External))
    }

    fn build_exit(llctx: &'tcx Context, module: &Module<'tcx>) -> FunctionValue<'tcx> {
        module.add_function(
            "exit",
            llctx.void_type().fn_type(&[llctx.i32_type().into()], false),
            Some(Linkage::External),
        )
    }

    fn build_abort(llctx: &'tcx Context, module: &Module<'tcx>) -> FunctionValue<'tcx> {
        module.add_function("abort", llctx.void_type().fn_type(&[], false), Some(Linkage::External))
    }

    fn build_iprintln(llctx: &'tcx Context, module: &Module<'tcx>) -> FunctionValue<'tcx> {
        let iprintln = module.add_function(
            "iprintln",
            llctx.void_type().fn_type(&[llctx.i64_type().into()], false),
            None,
        );
        // TODO
        iprintln
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

    fn build_rc_release(llctx: &'tcx Context, module: &Module<'tcx>) -> FunctionValue<'tcx> {
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
