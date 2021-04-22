// use crate::*;
// use inkwell::builder::Builder;
// use inkwell::types::BasicType;
// use inkwell::{AddressSpace, IntPredicate};

// impl<'a, 'tcx> FnCtx<'a, 'tcx> {
//     /// builds generic `rc_retain`
//     crate fn build_rc_retain(&mut self, lvalue: LvalueRef<'tcx>) -> FunctionValue<'tcx> {
//         let name = format!("rc_retain<{}>", lvalue.ty.deref_ty());
//         if let Some(f) = self.module.get_function(&name) {
//             return f;
//         }
//         let llty = self
//             .llctx
//             .void_type()
//             .fn_type(&[self.llvm_ty(lvalue.ty).ptr_type(AddressSpace::Generic).into()], false);
//         let rc_retain = self.module.add_function(&name, llty, None);

//         let builder = self.llctx.create_builder();
//         let block = self.llctx.append_basic_block(rc_retain, "rc_retain_start");
//         builder.position_at_end(block);

//         let alloca_ptr = rc_retain.get_first_param().unwrap().into_pointer_value();

//         let (_, rc_ptr) = self.builder_get_box_pointers(&builder, lvalue, alloca_ptr);

//         let refcount = builder.build_load(rc_ptr, "load_rc").into_int_value();
//         let increment = builder.build_int_add(refcount, self.vals.one32, "increment_rc");
//         builder.build_store(rc_ptr, increment);

//         builder.build_return(None);
//         rc_retain
//     }

//     /// builds generic `rc_release`
//     crate fn build_rc_release(&mut self, lvalue: LvalueRef<'tcx>) -> FunctionValue<'tcx> {
//         let name = format!("rc_release<{}>", lvalue.ty.deref_ty());
//         if let Some(f) = self.module.get_function(&name) {
//             return f;
//         }
//         let llty = self
//             .llctx
//             .void_type()
//             .fn_type(&[self.llvm_ty(lvalue.ty).ptr_type(AddressSpace::Generic).into()], false);
//         let rc_release = self.module.add_function(&name, llty, None);

//         let builder = self.llctx.create_builder();
//         let block = self.llctx.append_basic_block(rc_release, "rc_release_start");
//         let then_block = self.llctx.append_basic_block(rc_release, "rc_release_free");
//         let else_block = self.llctx.append_basic_block(rc_release, "rc_release_ret");
//         builder.position_at_end(block);

//         let alloca_ptr = rc_release.get_first_param().unwrap().into_pointer_value();

//         // self.build_print_str(&builder, "rc_release_count\n");

//         let (_cast, rc_ptr) = self.builder_get_box_pointers(&builder, lvalue, alloca_ptr);
//         let refcount = builder.build_load(rc_ptr, "load_rc").into_int_value();

//         let dec = builder.build_int_sub(refcount, self.vals.one32, "decrement");
//         builder.build_store(rc_ptr, dec);

//         // builder.build_call(
//         //     self.native_functions.print,
//         //     &[builder.build_int_cast(dec, self.types.int, "rc_release_count").into()],
//         //     "print_rc",
//         // );

//         let cmp = builder.build_int_compare(IntPredicate::EQ, dec, self.vals.zero32, "rc_cmp");
//         builder.build_conditional_branch(cmp, then_block, else_block);

//         builder.position_at_end(then_block);
//         // builder.build_free(cast);
//         builder.build_return(None);

//         builder.position_at_end(else_block);
//         builder.build_return(None);
//         rc_release
//     }

//     fn builder_get_box_pointers(
//         &self,
//         builder: &Builder<'tcx>,
//         lvalue: LvalueRef<'tcx>,
//         alloca_ptr: PointerValue<'tcx>,
//     ) -> (PointerValue<'tcx>, PointerValue<'tcx>) {
//         let malloc_ptr = builder.build_load(alloca_ptr, "load_box").into_pointer_value();

//         let boxed_ptr_ty = self.llvm_boxed_ty(lvalue.ty.deref_ty()).ptr_type(AddressSpace::Generic);
//         let cast = builder.build_pointer_cast(malloc_ptr, boxed_ptr_ty, "rc_retain_box_cast");
//         let rc_ptr = builder.build_struct_gep(cast, 1, "rc").unwrap();
//         (cast, rc_ptr)
//     }

//     crate fn build_print_str(&self, builder: &Builder<'tcx>, string: &str) {
//         let string = self.llctx.const_string(string.as_bytes(), true);
//         let alloca = builder.build_alloca(string.get_type(), "alloc_str");
//         builder.build_store(alloca, string);
//         let bitcast = builder.build_bitcast(alloca, self.types.i8ptr, "cast_str");
//         builder.build_call(self.native_functions.printf, &[bitcast], "print_str");
//     }
// }
