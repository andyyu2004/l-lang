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
  %tmp2 = alloca %opaque.3*
  %tmp3 = alloca %opaque.3*
  %tmp4 = alloca %opaque.3*
  %tmp5 = alloca %opaque.3*
  %expr = alloca %opaque.3*
  %malloccall = tail call i8* @malloc(i32 ptrtoint (<{ %opaque.3, i32 }>* getelementptr (<{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to <{ %opaque.3, i32 }>*
  %rc = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %box, i32 0, i32 1
  %0 = atomicrmw xchg i32* %rc, i32 1 seq_cst
  %rc_gep = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %box, i32 0, i32 0
  store %opaque.3* %rc_gep, %opaque.3** %tmp1
  %load_box = load %opaque.3*, %opaque.3** %tmp1
  %rc_cast = bitcast %opaque.3* %load_box to <{ %opaque.3, i32 }>*
  %rc6 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast, i32 0, i32 1
  %1 = atomicrmw add i32* %rc6, i32 1 seq_cst
  %malloccall7 = tail call i8* @malloc(i32 ptrtoint (<{ %opaque.3, i32 }>* getelementptr (<{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* null, i32 1) to i32))
  %box8 = bitcast i8* %malloccall7 to <{ %opaque.3, i32 }>*
  %rc9 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %box8, i32 0, i32 1
  %2 = atomicrmw xchg i32* %rc9, i32 1 seq_cst
  %rc_gep10 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %box8, i32 0, i32 0
  store %opaque.3* %rc_gep10, %opaque.3** %tmp3
  %load_box11 = load %opaque.3*, %opaque.3** %tmp3
  %rc_cast12 = bitcast %opaque.3* %load_box11 to <{ %opaque.3, i32 }>*
  %rc13 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast12, i32 0, i32 1
  %3 = atomicrmw add i32* %rc13, i32 1 seq_cst
  %fcall = call %opaque.3 @"Expr::Int"(i64 5)
  %load_deref = load %opaque.3*, %opaque.3** %tmp3
  store %opaque.3 %fcall, %opaque.3* %load_deref
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  %load = load %opaque.3*, %opaque.3** %tmp3
  store %opaque.3* %load, %opaque.3** %tmp2
  %load_box14 = load %opaque.3*, %opaque.3** %tmp2
  %rc_cast15 = bitcast %opaque.3* %load_box14 to <{ %opaque.3, i32 }>*
  %rc16 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast15, i32 0, i32 1
  %4 = atomicrmw add i32* %rc16, i32 1 seq_cst
  %malloccall17 = tail call i8* @malloc(i32 ptrtoint (<{ %opaque.3, i32 }>* getelementptr (<{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* null, i32 1) to i32))
  %box18 = bitcast i8* %malloccall17 to <{ %opaque.3, i32 }>*
  %rc19 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %box18, i32 0, i32 1
  %5 = atomicrmw xchg i32* %rc19, i32 1 seq_cst
  %rc_gep20 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %box18, i32 0, i32 0
  store %opaque.3* %rc_gep20, %opaque.3** %tmp5
  %load_box21 = load %opaque.3*, %opaque.3** %tmp5
  %rc_cast22 = bitcast %opaque.3* %load_box21 to <{ %opaque.3, i32 }>*
  %rc23 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast22, i32 0, i32 1
  %6 = atomicrmw add i32* %rc23, i32 1 seq_cst
  %fcall24 = call %opaque.3 @"Expr::Int"(i64 9)
  %load_deref25 = load %opaque.3*, %opaque.3** %tmp5
  store %opaque.3 %fcall24, %opaque.3* %load_deref25
  br label %basic_block2

basic_block2:                                     ; preds = %basic_block1
  %load26 = load %opaque.3*, %opaque.3** %tmp5
  store %opaque.3* %load26, %opaque.3** %tmp4
  %load_box27 = load %opaque.3*, %opaque.3** %tmp4
  %rc_cast28 = bitcast %opaque.3* %load_box27 to <{ %opaque.3, i32 }>*
  %rc29 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast28, i32 0, i32 1
  %7 = atomicrmw add i32* %rc29, i32 1 seq_cst
  %load30 = load %opaque.3*, %opaque.3** %tmp2
  %load31 = load %opaque.3*, %opaque.3** %tmp4
  %fcall32 = call %opaque.3 @"Expr::Add"(%opaque.3* %load30, %opaque.3* %load31)
  %load_deref33 = load %opaque.3*, %opaque.3** %tmp1
  store %opaque.3 %fcall32, %opaque.3* %load_deref33
  br label %basic_block3

basic_block3:                                     ; preds = %basic_block2
  %load34 = load %opaque.3*, %opaque.3** %tmp1
  store %opaque.3* %load34, %opaque.3** %tmp
  %load_box35 = load %opaque.3*, %opaque.3** %tmp
  %rc_cast36 = bitcast %opaque.3* %load_box35 to <{ %opaque.3, i32 }>*
  %rc37 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast36, i32 0, i32 1
  %8 = atomicrmw add i32* %rc37, i32 1 seq_cst
  %load38 = load %opaque.3*, %opaque.3** %tmp
  store %opaque.3* %load38, %opaque.3** %expr
  %load_box39 = load %opaque.3*, %opaque.3** %expr
  %rc_cast40 = bitcast %opaque.3* %load_box39 to <{ %opaque.3, i32 }>*
  %rc41 = getelementptr inbounds <{ %opaque.3, i32 }>, <{ %opaque.3, i32 }>* %rc_cast40, i32 0, i32 1
  %9 = atomicrmw add i32* %rc41, i32 1 seq_cst
  %load42 = load %opaque.3*, %opaque.3** %expr
  %fcall43 = call i64 @eval(%opaque.3* %load42)
  store i64 %fcall43, i64* %retvar
  br label %basic_block4

basic_block4:                                     ; preds = %basic_block3
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
  %lvalue_pointer_cast = bitcast { %opaque.3*, %opaque.3* }* %struct_gep to { i64 }*
  %struct_gep16 = getelementptr inbounds { i64 }, { i64 }* %lvalue_pointer_cast, i32 0, i32 0
  %load17 = load i64, i64* %struct_gep16
  store i64 %load17, i64* %i
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
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_block6:                                     ; preds = %basic_block3
  unreachable

basic_block7:                                     ; preds = %basic_block4
  %load44 = load %opaque.3*, %opaque.3** %r
  %fcall45 = call i64 @eval(%opaque.3* %load44)
  store i64 %fcall45, i64* %tmp11
  br label %basic_block8

basic_block8:                                     ; preds = %basic_block7
  %load46 = load i64, i64* %tmp10
  %load47 = load i64, i64* %tmp11
  %iadd = add i64 %load46, %load47
  store i64 %iadd, i64* %retvar
  br label %basic_block5
}

declare noalias i8* @malloc(i32)
