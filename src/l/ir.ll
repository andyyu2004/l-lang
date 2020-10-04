; ModuleID = 'main'
source_filename = "main"

%opaque = type { i64, { i64 } }
%opaque.0 = type { i64, { %opaque.1* } }
%opaque.1 = type { i64, %opaque.0 }
%opaque.2 = type { i64, { i64 } }
%opaque.3 = type { i64, { %opaque.3*, %opaque.3* } }

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

define %opaque @"Option::Some"(i64 %0) {
basic_block0:
  %retvar = alloca %opaque
  %1 = alloca i64
  store i64 %0, i64* %1
  %discr_gep = getelementptr inbounds %opaque, %opaque* %retvar, i32 0, i32 0
  store i64 1, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque, %opaque* %retvar, i32 0, i32 1
  %load = load i64, i64* %1
  %enum_content_gep = getelementptr inbounds { i64 }, { i64 }* %enum_gep, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load_ret = load %opaque, %opaque* %retvar
  ret %opaque %load_ret
}

define %opaque.0 @"NodeOption::Some"(%opaque.1* %0) {
basic_block0:
  %retvar = alloca %opaque.0
  %1 = alloca %opaque.1*
  store %opaque.1* %0, %opaque.1** %1
  %discr_gep = getelementptr inbounds %opaque.0, %opaque.0* %retvar, i32 0, i32 0
  store i64 1, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque.0, %opaque.0* %retvar, i32 0, i32 1
  %load = load %opaque.1*, %opaque.1** %1
  %enum_content_gep = getelementptr inbounds { %opaque.1* }, { %opaque.1* }* %enum_gep, i32 0, i32 0
  store %opaque.1* %load, %opaque.1** %enum_content_gep
  %load_ret = load %opaque.0, %opaque.0* %retvar
  ret %opaque.0 %load_ret
}

define %opaque.2 @"Either::Left"(i64 %0) {
basic_block0:
  %retvar = alloca %opaque.2
  %1 = alloca i64
  store i64 %0, i64* %1
  %discr_gep = getelementptr inbounds %opaque.2, %opaque.2* %retvar, i32 0, i32 0
  store i64 0, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque.2, %opaque.2* %retvar, i32 0, i32 1
  %load = load i64, i64* %1
  %enum_content_gep = getelementptr inbounds { i64 }, { i64 }* %enum_gep, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load_ret = load %opaque.2, %opaque.2* %retvar
  ret %opaque.2 %load_ret
}

define %opaque.2 @"Either::Right"(i64 %0) {
basic_block0:
  %retvar = alloca %opaque.2
  %1 = alloca i64
  store i64 %0, i64* %1
  %discr_gep = getelementptr inbounds %opaque.2, %opaque.2* %retvar, i32 0, i32 0
  store i64 1, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque.2, %opaque.2* %retvar, i32 0, i32 1
  %load = load i64, i64* %1
  %enum_content_gep = getelementptr inbounds { i64 }, { i64 }* %enum_gep, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load_ret = load %opaque.2, %opaque.2* %retvar
  ret %opaque.2 %load_ret
}

define %opaque.3 @"Expr::Int"(i64 %0) {
basic_block0:
  %retvar = alloca %opaque.3
  %1 = alloca i64
  store i64 %0, i64* %1
  %discr_gep = getelementptr inbounds %opaque.3, %opaque.3* %retvar, i32 0, i32 0
  store i64 0, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque.3, %opaque.3* %retvar, i32 0, i32 1
  %enum_ptr_cast = bitcast { %opaque.3*, %opaque.3* }* %enum_gep to { i64 }*
  %load = load i64, i64* %1
  %enum_content_gep = getelementptr inbounds { i64 }, { i64 }* %enum_ptr_cast, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load_ret = load %opaque.3, %opaque.3* %retvar
  ret %opaque.3 %load_ret
}

define %opaque.3 @"Expr::Add"(%opaque.3* %0, %opaque.3* %1) {
basic_block0:
  %retvar = alloca %opaque.3
  %2 = alloca %opaque.3*
  store %opaque.3* %0, %opaque.3** %2
  %3 = alloca %opaque.3*
  store %opaque.3* %1, %opaque.3** %3
  %discr_gep = getelementptr inbounds %opaque.3, %opaque.3* %retvar, i32 0, i32 0
  store i64 1, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque.3, %opaque.3* %retvar, i32 0, i32 1
  %load = load %opaque.3*, %opaque.3** %2
  %enum_content_gep = getelementptr inbounds { %opaque.3*, %opaque.3* }, { %opaque.3*, %opaque.3* }* %enum_gep, i32 0, i32 0
  store %opaque.3* %load, %opaque.3** %enum_content_gep
  %load1 = load %opaque.3*, %opaque.3** %3
  %enum_content_gep2 = getelementptr inbounds { %opaque.3*, %opaque.3* }, { %opaque.3*, %opaque.3* }* %enum_gep, i32 0, i32 1
  store %opaque.3* %load1, %opaque.3** %enum_content_gep2
  %load_ret = load %opaque.3, %opaque.3* %retvar
  ret %opaque.3 %load_ret
}

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca %opaque.3*
  %tmp1 = alloca %opaque.3*
  %expr = alloca %opaque.3*
  %malloccall = tail call i8* @malloc(i32 ptrtoint (<{ %opaque.3, i32 }>* getelementptr (<{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to <{ %opaque.3, i32 }>*
  %rc = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %box, i32 0, i32 1
  %0 = atomicrmw xchg i32* %rc, i32 1 seq_cst
  %rc_gep = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %box, i32 0, i32 0
  store %opaque.3* %rc_gep, %opaque.3** %tmp1
  %load_box = load %opaque.3*, %opaque.3** %tmp1
  %rc_cast = bitcast %opaque.3* %load_box to <{ %opaque.3, i32 }>*
  %rc2 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast, i32 0, i32 1
  %1 = atomicrmw add i32* %rc2, i32 1 seq_cst
  %fcall = call %opaque.3 @"Expr::Int"(i64 5)
  %load_deref = load %opaque.3*, %opaque.3** %tmp1
  store %opaque.3 %fcall, %opaque.3* %load_deref
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  %load = load %opaque.3*, %opaque.3** %tmp1
  store %opaque.3* %load, %opaque.3** %tmp
  %load_box3 = load %opaque.3*, %opaque.3** %tmp
  %rc_cast4 = bitcast %opaque.3* %load_box3 to <{ %opaque.3, i32 }>*
  %rc5 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast4, i32 0, i32 1
  %2 = atomicrmw add i32* %rc5, i32 1 seq_cst
  %load6 = load %opaque.3*, %opaque.3** %tmp
  store %opaque.3* %load6, %opaque.3** %expr
  %load_box7 = load %opaque.3*, %opaque.3** %expr
  %rc_cast8 = bitcast %opaque.3* %load_box7 to <{ %opaque.3, i32 }>*
  %rc9 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast8, i32 0, i32 1
  %3 = atomicrmw add i32* %rc9, i32 1 seq_cst
  %load10 = load %opaque.3*, %opaque.3** %expr
  %fcall11 = call i64 @eval(%opaque.3* %load10)
  store i64 %fcall11, i64* %retvar
  br label %basic_block2

basic_block2:                                     ; preds = %basic_block1
  %rc_release_cast = bitcast %opaque.3** %tmp1 to i8*
  %load_box12 = load %opaque.3*, %opaque.3** %tmp1
  %rc_cast13 = bitcast %opaque.3* %load_box12 to <{ %opaque.3, i32 }>*
  %rc14 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast13, i32 0, i32 1
  call void @rc_release(i8* %rc_release_cast, i32* %rc14)
  %rc_release_cast15 = bitcast %opaque.3** %tmp to i8*
  %load_box16 = load %opaque.3*, %opaque.3** %tmp
  %rc_cast17 = bitcast %opaque.3* %load_box16 to <{ %opaque.3, i32 }>*
  %rc18 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast17, i32 0, i32 1
  call void @rc_release(i8* %rc_release_cast15, i32* %rc18)
  %rc_release_cast19 = bitcast %opaque.3** %expr to i8*
  %load_box20 = load %opaque.3*, %opaque.3** %expr
  %rc_cast21 = bitcast %opaque.3* %load_box20 to <{ %opaque.3, i32 }>*
  %rc22 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast21, i32 0, i32 1
  call void @rc_release(i8* %rc_release_cast19, i32* %rc22)
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

define i64 @eval(%opaque.3* %0) {
basic_block0:
  %retvar = alloca i64
  %expr = alloca %opaque.3*
  store %opaque.3* %0, %opaque.3** %expr
  %expr1 = alloca %opaque.3*
  %tmp = alloca i1
  %tmp2 = alloca i64
  %tmp3 = alloca i1
  %tmp4 = alloca i1
  %i = alloca i64
  %tmp5 = alloca i1
  %tmp6 = alloca i64
  %tmp7 = alloca i1
  %tmp8 = alloca i1
  %l = alloca %opaque.3*
  %tmp9 = alloca i1
  %r = alloca %opaque.3*
  %tmp10 = alloca i64
  %tmp11 = alloca i64
  %load = load %opaque.3*, %opaque.3** %expr
  store %opaque.3* %load, %opaque.3** %expr1
  %load_box = load %opaque.3*, %opaque.3** %expr1
  %rc_cast = bitcast %opaque.3* %load_box to <{ %opaque.3, i32 }>*
  %rc = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast, i32 0, i32 1
  %1 = atomicrmw add i32* %rc, i32 1 seq_cst
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  store i1 true, i1* %tmp
  %load_deref = load %opaque.3*, %opaque.3** %expr1
  %discr_gep = getelementptr inbounds %opaque.3, %opaque.3* %load_deref, i32 0, i32 0
  %load_discr = load i64, i64* %discr_gep
  store i64 %load_discr, i64* %tmp2
  %load12 = load i64, i64* %tmp2
  %icmp_eq = icmp eq i64 0, %load12
  store i1 %icmp_eq, i1* %tmp3
  %load13 = load i1, i1* %tmp3
  %load14 = load i1, i1* %tmp
  %and = and i1 %load13, %load14
  store i1 %and, i1* %tmp
  store i1 true, i1* %tmp4
  %load_deref15 = load %opaque.3*, %opaque.3** %expr1
  %struct_gep = getelementptr inbounds %opaque.3, %opaque.3* %load_deref15, i32 0, i32 1
  %struct_gep16 = getelementptr inbounds { %opaque.3*, %opaque.3* }, { %opaque.3*, %opaque.3* }* %struct_gep, i32 0, i32 0
  %load17 = load %opaque.3*, %opaque.3** %struct_gep16
  store %opaque.3* %load17, i64* %i
  %load18 = load i1, i1* %tmp
  br i1 %load18, label %basic_block2, label %basic_block3

basic_block2:                                     ; preds = %basic_block1
  %load19 = load i64, i64* %i
  store i64 %load19, i64* %retvar
  br label %basic_block5

basic_block3:                                     ; preds = %basic_block1
  store i1 true, i1* %tmp5
  %load_deref20 = load %opaque.3*, %opaque.3** %expr1
  %discr_gep21 = getelementptr inbounds %opaque.3, %opaque.3* %load_deref20, i32 0, i32 0
  %load_discr22 = load i64, i64* %discr_gep21
  store i64 %load_discr22, i64* %tmp6
  %load23 = load i64, i64* %tmp6
  %icmp_eq24 = icmp eq i64 1, %load23
  store i1 %icmp_eq24, i1* %tmp7
  %load25 = load i1, i1* %tmp7
  %load26 = load i1, i1* %tmp5
  %and27 = and i1 %load25, %load26
  store i1 %and27, i1* %tmp5
  store i1 true, i1* %tmp8
  %load_deref28 = load %opaque.3*, %opaque.3** %expr1
  %struct_gep29 = getelementptr inbounds %opaque.3, %opaque.3* %load_deref28, i32 0, i32 1
  %struct_gep30 = getelementptr inbounds { %opaque.3*, %opaque.3* }, { %opaque.3*, %opaque.3* }* %struct_gep29, i32 0, i32 0
  %load31 = load %opaque.3*, %opaque.3** %struct_gep30
  store %opaque.3* %load31, %opaque.3** %l
  %load_box32 = load %opaque.3*, %opaque.3** %l
  %rc_cast33 = bitcast %opaque.3* %load_box32 to <{ %opaque.3, i32 }>*
  %rc34 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast33, i32 0, i32 1
  %2 = atomicrmw add i32* %rc34, i32 1 seq_cst
  store i1 true, i1* %tmp9
  %load_deref35 = load %opaque.3*, %opaque.3** %expr1
  %struct_gep36 = getelementptr inbounds %opaque.3, %opaque.3* %load_deref35, i32 0, i32 1
  %struct_gep37 = getelementptr inbounds { %opaque.3*, %opaque.3* }, { %opaque.3*, %opaque.3* }* %struct_gep36, i32 0, i32 1
  %load38 = load %opaque.3*, %opaque.3** %struct_gep37
  store %opaque.3* %load38, %opaque.3** %r
  %load_box39 = load %opaque.3*, %opaque.3** %r
  %rc_cast40 = bitcast %opaque.3* %load_box39 to <{ %opaque.3, i32 }>*
  %rc41 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast40, i32 0, i32 1
  %3 = atomicrmw add i32* %rc41, i32 1 seq_cst
  %load42 = load i1, i1* %tmp5
  br i1 %load42, label %basic_block4, label %basic_block6

basic_block4:                                     ; preds = %basic_block3
  %load43 = load %opaque.3*, %opaque.3** %l
  %fcall = call i64 @eval(%opaque.3* %load43)
  store i64 %fcall, i64* %tmp10
  br label %basic_block7

basic_block5:                                     ; preds = %basic_block8, %basic_block2
  %rc_release_cast = bitcast %opaque.3** %l to i8*
  %load_box44 = load %opaque.3*, %opaque.3** %l
  %rc_cast45 = bitcast %opaque.3* %load_box44 to <{ %opaque.3, i32 }>*
  %rc46 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast45, i32 0, i32 1
  call void @rc_release(i8* %rc_release_cast, i32* %rc46)
  %rc_release_cast47 = bitcast %opaque.3** %r to i8*
  %load_box48 = load %opaque.3*, %opaque.3** %r
  %rc_cast49 = bitcast %opaque.3* %load_box48 to <{ %opaque.3, i32 }>*
  %rc50 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast49, i32 0, i32 1
  call void @rc_release(i8* %rc_release_cast47, i32* %rc50)
  %rc_release_cast51 = bitcast %opaque.3** %expr1 to i8*
  %load_box52 = load %opaque.3*, %opaque.3** %expr1
  %rc_cast53 = bitcast %opaque.3* %load_box52 to <{ %opaque.3, i32 }>*
  %rc54 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast53, i32 0, i32 1
  call void @rc_release(i8* %rc_release_cast51, i32* %rc54)
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_block6:                                     ; preds = %basic_block3
  unreachable

basic_block7:                                     ; preds = %basic_block4
  %load55 = load %opaque.3*, %opaque.3** %r
  %fcall56 = call i64 @eval(%opaque.3* %load55)
  store i64 %fcall56, i64* %tmp11
  br label %basic_block8

basic_block8:                                     ; preds = %basic_block7
  %load57 = load i64, i64* %tmp10
  %load58 = load i64, i64* %tmp11
  %iadd = add i64 %load57, %load58
  store i64 %iadd, i64* %retvar
  br label %basic_block5
}

declare noalias i8* @malloc(i32)
