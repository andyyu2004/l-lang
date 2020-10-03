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

define { i64, <{ i64 }> } @"Option::Some"(i64 %0) {
basic_block0:
  %retvar = alloca { i64, <{ i64 }> }
  %1 = alloca i64
  store i64 %0, i64* %1
  %discr_gep = getelementptr inbounds { i64, <{ i64 }> }, { i64, <{ i64 }> }* %retvar, i32 0, i32 0
  store i64 0, i64* %discr_gep
  %enum_gep = getelementptr inbounds { i64, <{ i64 }> }, { i64, <{ i64 }> }* %retvar, i32 0, i32 1
  %load = load i64, i64* %1
  %enum_content_gep = getelementptr inbounds <{ i64 }>, <{ i64 }>* %enum_gep, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load_ret = load { i64, <{ i64 }> }, { i64, <{ i64 }> }* %retvar
  ret { i64, <{ i64 }> } %load_ret
}

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca { i64, <{ i64 }> }
  %opt = alloca { i64, <{ i64 }> }
  %tmp1 = alloca { i64, <{ i64 }> }
  %opt2 = alloca { i64, <{ i64 }> }
  %tmp3 = alloca i1
  %tmp4 = alloca i64
  %tmp5 = alloca i1
  %tmp6 = alloca i1
  %tmp7 = alloca i64
  %tmp8 = alloca i1
  %discr_gep = getelementptr inbounds { i64, <{ i64 }> }, { i64, <{ i64 }> }* %tmp, i32 0, i32 0
  store i64 1, i64* %discr_gep
  %enum_gep = getelementptr inbounds { i64, <{ i64 }> }, { i64, <{ i64 }> }* %tmp, i32 0, i32 1
  %enum_ptr_cast = bitcast <{ i64 }>* %enum_gep to <{}>*
  %load = load { i64, <{ i64 }> }, { i64, <{ i64 }> }* %tmp
  store { i64, <{ i64 }> } %load, { i64, <{ i64 }> }* %opt
  %fcall = call { i64, <{ i64 }> } @"Option::Some"(i64 5)
  store { i64, <{ i64 }> } %fcall, { i64, <{ i64 }> }* %tmp1
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  %load9 = load { i64, <{ i64 }> }, { i64, <{ i64 }> }* %tmp1
  store { i64, <{ i64 }> } %load9, { i64, <{ i64 }> }* %opt2
  br label %basic_block2

basic_block2:                                     ; preds = %basic_block1
  store i1 true, i1* %tmp3
  %discr_gep10 = getelementptr inbounds { i64, <{ i64 }> }, { i64, <{ i64 }> }* %opt2, i32 0, i32 0
  %load_discr = load i64, i64* %discr_gep10
  store i64 %load_discr, i64* %tmp4
  %load11 = load i64, i64* %tmp4
  %icmp_eq = icmp eq i64 0, %load11
  store i1 %icmp_eq, i1* %tmp5
  %load12 = load i1, i1* %tmp5
  %load13 = load i1, i1* %tmp3
  %and = and i1 %load12, %load13
  store i1 %and, i1* %tmp3
  %load14 = load i1, i1* %tmp3
  br i1 %load14, label %basic_block3, label %basic_block4

basic_block3:                                     ; preds = %basic_block2
  store i64 99, i64* %retvar
  br label %basic_block6

basic_block4:                                     ; preds = %basic_block2
  store i1 true, i1* %tmp6
  %discr_gep15 = getelementptr inbounds { i64, <{ i64 }> }, { i64, <{ i64 }> }* %opt2, i32 0, i32 0
  %load_discr16 = load i64, i64* %discr_gep15
  store i64 %load_discr16, i64* %tmp7
  %load17 = load i64, i64* %tmp7
  %icmp_eq18 = icmp eq i64 1, %load17
  store i1 %icmp_eq18, i1* %tmp8
  %load19 = load i1, i1* %tmp8
  %load20 = load i1, i1* %tmp6
  %and21 = and i1 %load19, %load20
  store i1 %and21, i1* %tmp6
  %load22 = load i1, i1* %tmp6
  br i1 %load22, label %basic_block5, label %basic_block7

basic_block5:                                     ; preds = %basic_block4
  store i64 77, i64* %retvar
  br label %basic_block6

basic_block6:                                     ; preds = %basic_block5, %basic_block3
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_block7:                                     ; preds = %basic_block4
  unreachable
}
