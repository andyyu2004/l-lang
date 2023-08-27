use inkwell::context::Context;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::values::*;
use inkwell::AddressSpace;
use std::ops::Deref;

pub struct NativeFunctions<'tcx> {
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
        let print = self.build_print();
        let print_addr = self.build_print_addr();
        let abort = self.build_abort();
        let exit = self.build_exit();
        NativeFunctions { abort, print, exit, print_addr, printf }
    }

    fn build_print_addr(&self) -> FunctionValue<'tcx> {
        let unit = self.struct_type(&[], false);
        let printfn = self.module.add_function(
            "print_addr",
            unit.fn_type(&[self.i8_type().ptr_type(AddressSpace::default()).into()], false),
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
            self.i8_type().ptr_type(AddressSpace::default()),
            "bitcast",
        );
        let printf = self.module.get_function("printf").unwrap();
        builder.build_call(printf, &[ptr.into(), param.into()], "printf");
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
            self.i8_type().ptr_type(AddressSpace::default()),
            "bitcast",
        );
        let printf = self.module.get_function("printf").unwrap();
        builder.build_call(printf, &[ptr.into(), param.into()], "printf");
        builder.build_return(Some(&self.const_struct(&[], false)));
        printfn
    }

    fn build_printf(&self) -> FunctionValue<'tcx> {
        self.module.add_function(
            "printf",
            self.i32_type()
                .fn_type(&[self.i8_type().ptr_type(AddressSpace::default()).into()], true),
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
