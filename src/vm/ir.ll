; ModuleID = 'main'
source_filename = "main"

define void @rc_release(i8* %0, i32* %1) {
rc_release:
  %2 = atomicrmw sub i32* %1, i32 1 seq_cst
  %rc_cmp = icmp ule i32 %2, 1
  br i1 %rc_cmp, label %free, label %ret

free:                                             ; preds = %rc_release
  tail call void @free(i8* %0)
  ret void

ret:                                              ; preds = %rc_release
  ret void
}

declare void @free(i8*)

declare void @iprintln(i64)

define { i8, <{ i64 }> } @"Option::Some"(i64 %0) {
basic_block0:
  %retvar = alloca { i8, <{ i64 }> }
  %1 = alloca i64
  store i64 %0, i64* %1
  %discr_gep = getelementptr inbounds { i8, <{ i64 }> }, { i8, <{ i64 }> }* %retvar, i32 0, i32 0
  store i8 0, i8* %discr_gep
  %enum_gep = getelementptr inbounds { i8, <{ i64 }> }, { i8, <{ i64 }> }* %retvar, i32 0, i32 1
  %load = load i64, i64* %1
  %enum_content_gep = getelementptr inbounds <{ i64 }>, <{ i64 }>* %enum_gep, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load_ret = load { i8, <{ i64 }> }, { i8, <{ i64 }> }* %retvar
  ret { i8, <{ i64 }> } %load_ret
}

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca { i8, <{ i64 }> }
  %x = alloca { i8, <{ i64 }> }
  %discr_gep = getelementptr inbounds { i8, <{ i64 }> }, { i8, <{ i64 }> }* %tmp, i32 0, i32 0
  store i8 1, i8* %discr_gep
  %enum_gep = getelementptr inbounds { i8, <{ i64 }> }, { i8, <{ i64 }> }* %tmp, i32 0, i32 1
  %enum_ptr_cast = bitcast <{ i64 }>* %enum_gep to <{}>*
  %load = load { i8, <{ i64 }> }, { i8, <{ i64 }> }* %tmp
  store { i8, <{ i64 }> } %load, { i8, <{ i64 }> }* %x
  store i64 9, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
