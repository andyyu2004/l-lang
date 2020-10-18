; ModuleID = 'main'
source_filename = "main"

%"Expr<>" = type { i64, { %"Expr<>"*, %"Expr<>"* } }

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

declare i32 @printf(i8*, ...)

define {} @print(i64 %0) {
printint:
  %alloca_str = alloca [4 x i8]
  store [4 x i8] c"%d\0A\00", [4 x i8]* %alloca_str
  %bitcast = bitcast [4 x i8]* %alloca_str to i8*
  %printf = call i32 (i8*, ...) @printf(i8* %bitcast, i64 %0)
  ret {} zeroinitializer
}

define {} @print_addr(i8* %0) {
printint:
  %alloca_str = alloca [4 x i8]
  store [4 x i8] c"%p\0A\00", [4 x i8]* %alloca_str
  %bitcast = bitcast [4 x i8]* %alloca_str to i8*
  %printf = call i32 (i8*, ...) @printf(i8* %bitcast, i8* %0)
  ret {} zeroinitializer
}

declare void @abort()

declare void @exit(i32)

define i64 @main() {
basic_blockbb0:
  %retvar = alloca i64
  %tmp = alloca %"Expr<>"*
  %tmp1 = alloca %"Expr<>"
  %tmp2 = alloca %"Expr<>"*
  %tmp3 = alloca %"Expr<>"
  %tmp4 = alloca %"Expr<>"*
  %tmp5 = alloca %"Expr<>"
  %expr = alloca %"Expr<>"*
  %tmp6 = alloca i64
  %fcall = call %"Expr<>" @"Expr::Int<>"(i64 5)
  store %"Expr<>" %fcall, %"Expr<>"* %tmp3
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load = load %"Expr<>", %"Expr<>"* %tmp3
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ %"Expr<>", i32 }* getelementptr ({ %"Expr<>", i32 }, { %"Expr<>", i32 }* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to { %"Expr<>", i32 }*
  %cast_malloc_ptr = bitcast { %"Expr<>", i32 }* %box to i8*
  %print_malloc_addr = call {} @print_addr(i8* %cast_malloc_ptr)
  %rc_gep = getelementptr inbounds { %"Expr<>", i32 }, { %"Expr<>", i32 }* %box, i32 0, i32 1
  store i32 0, i32* %rc_gep
  %box_gep = getelementptr inbounds { %"Expr<>", i32 }, { %"Expr<>", i32 }* %box, i32 0, i32 0
  store %"Expr<>" %load, %"Expr<>"* %box_gep
  store %"Expr<>"* %box_gep, %"Expr<>"** %tmp2
  call void @"rc_retain<Expr<>>"(%"Expr<>"** %tmp2)
  %fcall7 = call %"Expr<>" @"Expr::Int<>"(i64 9)
  store %"Expr<>" %fcall7, %"Expr<>"* %tmp5
  br label %basic_blockbb2

basic_blockbb2:                                   ; preds = %basic_blockbb1
  %load8 = load %"Expr<>", %"Expr<>"* %tmp5
  %malloccall9 = tail call i8* @malloc(i32 ptrtoint ({ %"Expr<>", i32 }* getelementptr ({ %"Expr<>", i32 }, { %"Expr<>", i32 }* null, i32 1) to i32))
  %box10 = bitcast i8* %malloccall9 to { %"Expr<>", i32 }*
  %cast_malloc_ptr11 = bitcast { %"Expr<>", i32 }* %box10 to i8*
  %print_malloc_addr12 = call {} @print_addr(i8* %cast_malloc_ptr11)
  %rc_gep13 = getelementptr inbounds { %"Expr<>", i32 }, { %"Expr<>", i32 }* %box10, i32 0, i32 1
  store i32 0, i32* %rc_gep13
  %box_gep14 = getelementptr inbounds { %"Expr<>", i32 }, { %"Expr<>", i32 }* %box10, i32 0, i32 0
  store %"Expr<>" %load8, %"Expr<>"* %box_gep14
  store %"Expr<>"* %box_gep14, %"Expr<>"** %tmp4
  call void @"rc_retain<Expr<>>"(%"Expr<>"** %tmp4)
  %load15 = load %"Expr<>"*, %"Expr<>"** %tmp2
  %load16 = load %"Expr<>"*, %"Expr<>"** %tmp4
  %fcall17 = call %"Expr<>" @"Expr::Add<>"(%"Expr<>"* %load15, %"Expr<>"* %load16)
  store %"Expr<>" %fcall17, %"Expr<>"* %tmp1
  br label %basic_blockbb3

basic_blockbb3:                                   ; preds = %basic_blockbb2
  %load18 = load %"Expr<>", %"Expr<>"* %tmp1
  %malloccall19 = tail call i8* @malloc(i32 ptrtoint ({ %"Expr<>", i32 }* getelementptr ({ %"Expr<>", i32 }, { %"Expr<>", i32 }* null, i32 1) to i32))
  %box20 = bitcast i8* %malloccall19 to { %"Expr<>", i32 }*
  %cast_malloc_ptr21 = bitcast { %"Expr<>", i32 }* %box20 to i8*
  %print_malloc_addr22 = call {} @print_addr(i8* %cast_malloc_ptr21)
  %rc_gep23 = getelementptr inbounds { %"Expr<>", i32 }, { %"Expr<>", i32 }* %box20, i32 0, i32 1
  store i32 0, i32* %rc_gep23
  %box_gep24 = getelementptr inbounds { %"Expr<>", i32 }, { %"Expr<>", i32 }* %box20, i32 0, i32 0
  store %"Expr<>" %load18, %"Expr<>"* %box_gep24
  store %"Expr<>"* %box_gep24, %"Expr<>"** %tmp
  call void @"rc_retain<Expr<>>"(%"Expr<>"** %tmp)
  %load25 = load %"Expr<>"*, %"Expr<>"** %tmp
  store %"Expr<>"* %load25, %"Expr<>"** %expr
  call void @"rc_retain<Expr<>>"(%"Expr<>"** %expr)
  %load26 = load %"Expr<>"*, %"Expr<>"** %expr
  %fcall27 = call i64 @"eval<>"(%"Expr<>"* %load26)
  store i64 %fcall27, i64* %tmp6
  br label %basic_blockbb4

basic_blockbb4:                                   ; preds = %basic_blockbb3
  store i64 8, i64* %retvar
  call void @"rc_release<Expr<>>"(%"Expr<>"** %expr)
  call void @"rc_release<Expr<>>"(%"Expr<>"** %tmp)
  call void @"rc_release<Expr<>>"(%"Expr<>"** %tmp4)
  call void @"rc_release<Expr<>>"(%"Expr<>"** %tmp2)
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

define i64 @"rc<int>"(i64* %0) {
rc_entry:
  %cast_box_ptr = bitcast i64* %0 to { i64, i64 }*
  %rc_gep = getelementptr inbounds { i64, i64 }, { i64, i64 }* %cast_box_ptr, i32 0, i32 1
  %load_refcount = load i64, i64* %rc_gep
  ret i64 %load_refcount
}

define %"Expr<>" @"Expr::Int<>"(i64 %0) {
basic_blockbb0:
  %retvar = alloca %"Expr<>"
  %1 = alloca i64
  store i64 %0, i64* %1
  %discr_gep = getelementptr inbounds %"Expr<>", %"Expr<>"* %retvar, i32 0, i32 0
  store i64 0, i64* %discr_gep
  %enum_gep = getelementptr inbounds %"Expr<>", %"Expr<>"* %retvar, i32 0, i32 1
  %enum_ptr_cast = bitcast { %"Expr<>"*, %"Expr<>"* }* %enum_gep to { i64 }*
  %load = load i64, i64* %1
  %enum_content_gep = getelementptr inbounds { i64 }, { i64 }* %enum_ptr_cast, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load_ret = load %"Expr<>", %"Expr<>"* %retvar
  ret %"Expr<>" %load_ret
}

define %"Expr<>" @"Expr::Add<>"(%"Expr<>"* %0, %"Expr<>"* %1) {
basic_blockbb0:
  %retvar = alloca %"Expr<>"
  %2 = alloca %"Expr<>"*
  store %"Expr<>"* %0, %"Expr<>"** %2
  %3 = alloca %"Expr<>"*
  store %"Expr<>"* %1, %"Expr<>"** %3
  %discr_gep = getelementptr inbounds %"Expr<>", %"Expr<>"* %retvar, i32 0, i32 0
  store i64 1, i64* %discr_gep
  %enum_gep = getelementptr inbounds %"Expr<>", %"Expr<>"* %retvar, i32 0, i32 1
  %load = load %"Expr<>"*, %"Expr<>"** %2
  %enum_content_gep = getelementptr inbounds { %"Expr<>"*, %"Expr<>"* }, { %"Expr<>"*, %"Expr<>"* }* %enum_gep, i32 0, i32 0
  store %"Expr<>"* %load, %"Expr<>"** %enum_content_gep
  %load1 = load %"Expr<>"*, %"Expr<>"** %3
  %enum_content_gep2 = getelementptr inbounds { %"Expr<>"*, %"Expr<>"* }, { %"Expr<>"*, %"Expr<>"* }* %enum_gep, i32 0, i32 1
  store %"Expr<>"* %load1, %"Expr<>"** %enum_content_gep2
  %load_ret = load %"Expr<>", %"Expr<>"* %retvar
  ret %"Expr<>" %load_ret
}

define i64 @"eval<>"(%"Expr<>"* %0) {
basic_blockbb0:
  %retvar = alloca i64
  %expr = alloca %"Expr<>"*
  store %"Expr<>"* %0, %"Expr<>"** %expr
  %tmp = alloca {}
  %tmp1 = alloca i64
  %tmp2 = alloca i1
  %tmp3 = alloca i64
  %tmp4 = alloca i1
  %i = alloca i64
  %tmp5 = alloca i1
  %tmp6 = alloca i64
  %tmp7 = alloca i1
  %l = alloca %"Expr<>"*
  %r = alloca %"Expr<>"*
  %tmp8 = alloca i64
  %tmp9 = alloca i64
  %load = load %"Expr<>"*, %"Expr<>"** %expr
  %fcall = call i64 @"rc<Expr<>>"(%"Expr<>"* %load)
  store i64 %fcall, i64* %tmp1
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load10 = load i64, i64* %tmp1
  %fcall11 = call {} @print(i64 %load10)
  store {} %fcall11, {}* %tmp
  br label %basic_blockbb2

basic_blockbb2:                                   ; preds = %basic_blockbb1
  br label %basic_blockbb3

basic_blockbb3:                                   ; preds = %basic_blockbb2
  store i1 true, i1* %tmp2
  %load_deref = load %"Expr<>"*, %"Expr<>"** %expr
  %discr_gep = getelementptr inbounds %"Expr<>", %"Expr<>"* %load_deref, i32 0, i32 0
  %load_discr = load i64, i64* %discr_gep
  store i64 %load_discr, i64* %tmp3
  %load12 = load i64, i64* %tmp3
  %icmp_eq = icmp eq i64 0, %load12
  store i1 %icmp_eq, i1* %tmp4
  %load13 = load i1, i1* %tmp4
  %load14 = load i1, i1* %tmp2
  %and = and i1 %load13, %load14
  store i1 %and, i1* %tmp2
  %load_deref15 = load %"Expr<>"*, %"Expr<>"** %expr
  %struct_gep = getelementptr inbounds %"Expr<>", %"Expr<>"* %load_deref15, i32 0, i32 1
  %lvalue_pointer_cast = bitcast { %"Expr<>"*, %"Expr<>"* }* %struct_gep to { i64 }*
  %struct_gep16 = getelementptr inbounds { i64 }, { i64 }* %lvalue_pointer_cast, i32 0, i32 0
  %load17 = load i64, i64* %struct_gep16
  store i64 %load17, i64* %i
  %load18 = load i1, i1* %tmp2
  br i1 %load18, label %basic_blockbb4, label %basic_blockbb5

basic_blockbb4:                                   ; preds = %basic_blockbb3
  %load19 = load i64, i64* %i
  store i64 %load19, i64* %retvar
  br label %basic_blockbb7

basic_blockbb5:                                   ; preds = %basic_blockbb3
  store i1 true, i1* %tmp5
  %load_deref20 = load %"Expr<>"*, %"Expr<>"** %expr
  %discr_gep21 = getelementptr inbounds %"Expr<>", %"Expr<>"* %load_deref20, i32 0, i32 0
  %load_discr22 = load i64, i64* %discr_gep21
  store i64 %load_discr22, i64* %tmp6
  %load23 = load i64, i64* %tmp6
  %icmp_eq24 = icmp eq i64 1, %load23
  store i1 %icmp_eq24, i1* %tmp7
  %load25 = load i1, i1* %tmp7
  %load26 = load i1, i1* %tmp5
  %and27 = and i1 %load25, %load26
  store i1 %and27, i1* %tmp5
  %load_deref28 = load %"Expr<>"*, %"Expr<>"** %expr
  %struct_gep29 = getelementptr inbounds %"Expr<>", %"Expr<>"* %load_deref28, i32 0, i32 1
  %struct_gep30 = getelementptr inbounds { %"Expr<>"*, %"Expr<>"* }, { %"Expr<>"*, %"Expr<>"* }* %struct_gep29, i32 0, i32 0
  %load31 = load %"Expr<>"*, %"Expr<>"** %struct_gep30
  store %"Expr<>"* %load31, %"Expr<>"** %l
  call void @"rc_retain<Expr<>>"(%"Expr<>"** %l)
  %load_deref32 = load %"Expr<>"*, %"Expr<>"** %expr
  %struct_gep33 = getelementptr inbounds %"Expr<>", %"Expr<>"* %load_deref32, i32 0, i32 1
  %struct_gep34 = getelementptr inbounds { %"Expr<>"*, %"Expr<>"* }, { %"Expr<>"*, %"Expr<>"* }* %struct_gep33, i32 0, i32 1
  %load35 = load %"Expr<>"*, %"Expr<>"** %struct_gep34
  store %"Expr<>"* %load35, %"Expr<>"** %r
  call void @"rc_retain<Expr<>>"(%"Expr<>"** %r)
  %load36 = load i1, i1* %tmp5
  br i1 %load36, label %basic_blockbb6, label %basic_blockbb8

basic_blockbb6:                                   ; preds = %basic_blockbb5
  %load37 = load %"Expr<>"*, %"Expr<>"** %l
  %fcall38 = call i64 @"eval<>"(%"Expr<>"* %load37)
  store i64 %fcall38, i64* %tmp8
  br label %basic_blockbb9

basic_blockbb7:                                   ; preds = %basic_blockbb10, %basic_blockbb4
  call void @"rc_release<Expr<>>"(%"Expr<>"** %r)
  call void @"rc_release<Expr<>>"(%"Expr<>"** %l)
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_blockbb8:                                   ; preds = %basic_blockbb5
  call void @exit(i32 1)
  unreachable

basic_blockbb9:                                   ; preds = %basic_blockbb6
  %load39 = load %"Expr<>"*, %"Expr<>"** %r
  %fcall40 = call i64 @"eval<>"(%"Expr<>"* %load39)
  store i64 %fcall40, i64* %tmp9
  br label %basic_blockbb10

basic_blockbb10:                                  ; preds = %basic_blockbb9
  %load41 = load i64, i64* %tmp8
  %load42 = load i64, i64* %tmp9
  %iadd = add i64 %load41, %load42
  store i64 %iadd, i64* %retvar
  br label %basic_blockbb7
}

define i64 @"rc<Expr<>>"(%"Expr<>"* %0) {
rc_entry:
  %cast_box_ptr = bitcast %"Expr<>"* %0 to { %"Expr<>", i64 }*
  %rc_gep = getelementptr inbounds { %"Expr<>", i64 }, { %"Expr<>", i64 }* %cast_box_ptr, i32 0, i32 1
  %load_refcount = load i64, i64* %rc_gep
  ret i64 %load_refcount
}

define i64 @"eval<>.1"(%"Expr<>"* %0) {
basic_blockbb0:
  %retvar = alloca i64
  %expr = alloca %"Expr<>"*
  store %"Expr<>"* %0, %"Expr<>"** %expr
  %tmp = alloca {}
  %tmp1 = alloca i64
  %tmp2 = alloca i1
  %tmp3 = alloca i64
  %tmp4 = alloca i1
  %i = alloca i64
  %tmp5 = alloca i1
  %tmp6 = alloca i64
  %tmp7 = alloca i1
  %l = alloca %"Expr<>"*
  %r = alloca %"Expr<>"*
  %tmp8 = alloca i64
  %tmp9 = alloca i64
  %load = load %"Expr<>"*, %"Expr<>"** %expr
  %fcall = call i64 @"rc<Expr<>>"(%"Expr<>"* %load)
  store i64 %fcall, i64* %tmp1
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load10 = load i64, i64* %tmp1
  %fcall11 = call {} @print(i64 %load10)
  store {} %fcall11, {}* %tmp
  br label %basic_blockbb2

basic_blockbb2:                                   ; preds = %basic_blockbb1
  br label %basic_blockbb3

basic_blockbb3:                                   ; preds = %basic_blockbb2
  store i1 true, i1* %tmp2
  %load_deref = load %"Expr<>"*, %"Expr<>"** %expr
  %discr_gep = getelementptr inbounds %"Expr<>", %"Expr<>"* %load_deref, i32 0, i32 0
  %load_discr = load i64, i64* %discr_gep
  store i64 %load_discr, i64* %tmp3
  %load12 = load i64, i64* %tmp3
  %icmp_eq = icmp eq i64 0, %load12
  store i1 %icmp_eq, i1* %tmp4
  %load13 = load i1, i1* %tmp4
  %load14 = load i1, i1* %tmp2
  %and = and i1 %load13, %load14
  store i1 %and, i1* %tmp2
  %load_deref15 = load %"Expr<>"*, %"Expr<>"** %expr
  %struct_gep = getelementptr inbounds %"Expr<>", %"Expr<>"* %load_deref15, i32 0, i32 1
  %lvalue_pointer_cast = bitcast { %"Expr<>"*, %"Expr<>"* }* %struct_gep to { i64 }*
  %struct_gep16 = getelementptr inbounds { i64 }, { i64 }* %lvalue_pointer_cast, i32 0, i32 0
  %load17 = load i64, i64* %struct_gep16
  store i64 %load17, i64* %i
  %load18 = load i1, i1* %tmp2
  br i1 %load18, label %basic_blockbb4, label %basic_blockbb5

basic_blockbb4:                                   ; preds = %basic_blockbb3
  %load19 = load i64, i64* %i
  store i64 %load19, i64* %retvar
  br label %basic_blockbb7

basic_blockbb5:                                   ; preds = %basic_blockbb3
  store i1 true, i1* %tmp5
  %load_deref20 = load %"Expr<>"*, %"Expr<>"** %expr
  %discr_gep21 = getelementptr inbounds %"Expr<>", %"Expr<>"* %load_deref20, i32 0, i32 0
  %load_discr22 = load i64, i64* %discr_gep21
  store i64 %load_discr22, i64* %tmp6
  %load23 = load i64, i64* %tmp6
  %icmp_eq24 = icmp eq i64 1, %load23
  store i1 %icmp_eq24, i1* %tmp7
  %load25 = load i1, i1* %tmp7
  %load26 = load i1, i1* %tmp5
  %and27 = and i1 %load25, %load26
  store i1 %and27, i1* %tmp5
  %load_deref28 = load %"Expr<>"*, %"Expr<>"** %expr
  %struct_gep29 = getelementptr inbounds %"Expr<>", %"Expr<>"* %load_deref28, i32 0, i32 1
  %struct_gep30 = getelementptr inbounds { %"Expr<>"*, %"Expr<>"* }, { %"Expr<>"*, %"Expr<>"* }* %struct_gep29, i32 0, i32 0
  %load31 = load %"Expr<>"*, %"Expr<>"** %struct_gep30
  store %"Expr<>"* %load31, %"Expr<>"** %l
  call void @"rc_retain<Expr<>>"(%"Expr<>"** %l)
  %load_deref32 = load %"Expr<>"*, %"Expr<>"** %expr
  %struct_gep33 = getelementptr inbounds %"Expr<>", %"Expr<>"* %load_deref32, i32 0, i32 1
  %struct_gep34 = getelementptr inbounds { %"Expr<>"*, %"Expr<>"* }, { %"Expr<>"*, %"Expr<>"* }* %struct_gep33, i32 0, i32 1
  %load35 = load %"Expr<>"*, %"Expr<>"** %struct_gep34
  store %"Expr<>"* %load35, %"Expr<>"** %r
  call void @"rc_retain<Expr<>>"(%"Expr<>"** %r)
  %load36 = load i1, i1* %tmp5
  br i1 %load36, label %basic_blockbb6, label %basic_blockbb8

basic_blockbb6:                                   ; preds = %basic_blockbb5
  %load37 = load %"Expr<>"*, %"Expr<>"** %l
  %fcall38 = call i64 @"eval<>"(%"Expr<>"* %load37)
  store i64 %fcall38, i64* %tmp8
  br label %basic_blockbb9

basic_blockbb7:                                   ; preds = %basic_blockbb10, %basic_blockbb4
  call void @"rc_release<Expr<>>"(%"Expr<>"** %r)
  call void @"rc_release<Expr<>>"(%"Expr<>"** %l)
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_blockbb8:                                   ; preds = %basic_blockbb5
  call void @exit(i32 1)
  unreachable

basic_blockbb9:                                   ; preds = %basic_blockbb6
  %load39 = load %"Expr<>"*, %"Expr<>"** %r
  %fcall40 = call i64 @"eval<>"(%"Expr<>"* %load39)
  store i64 %fcall40, i64* %tmp9
  br label %basic_blockbb10

basic_blockbb10:                                  ; preds = %basic_blockbb9
  %load41 = load i64, i64* %tmp8
  %load42 = load i64, i64* %tmp9
  %iadd = add i64 %load41, %load42
  store i64 %iadd, i64* %retvar
  br label %basic_blockbb7
}

define {} @"take_box<>"(i64* %0) {
basic_blockbb0:
  %retvar = alloca {}
  %i = alloca i64*
  store i64* %0, i64** %i
  %tmp = alloca {}
  %tmp1 = alloca i64
  %load = load i64*, i64** %i
  %fcall = call i64 @"rc<int>"(i64* %load)
  store i64 %fcall, i64* %tmp1
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load2 = load i64, i64* %tmp1
  %fcall3 = call {} @print(i64 %load2)
  store {} %fcall3, {}* %tmp
  br label %basic_blockbb2

basic_blockbb2:                                   ; preds = %basic_blockbb1
  store {} undef, {}* %retvar
  %load_ret = load {}, {}* %retvar
  ret {} %load_ret
}

declare noalias i8* @malloc(i32)

define void @"rc_retain<Expr<>>"(%"Expr<>"** %0) {
rc_retain_start:
  %load_box = load %"Expr<>"*, %"Expr<>"** %0
  %cast_malloc_ptr = bitcast %"Expr<>"* %load_box to i8*
  %print_malloc_addr = call {} @print_addr(i8* %cast_malloc_ptr)
  %rc_retain_box_cast = bitcast %"Expr<>"* %load_box to { %"Expr<>", i32 }*
  %rc = getelementptr inbounds { %"Expr<>", i32 }, { %"Expr<>", i32 }* %rc_retain_box_cast, i32 0, i32 1
  %load_rc = load i32, i32* %rc
  %increment_rc = add i32 %load_rc, 1
  store i32 %increment_rc, i32* %rc
  %i64rc = sext i32 %increment_rc to i64
  %print_rc = call {} @print(i64 %i64rc)
  ret void
}

define void @"rc_release<Expr<>>"(%"Expr<>"** %0) {
rc_release_start:
  %load_box = load %"Expr<>"*, %"Expr<>"** %0
  %rc_release_box_cast = bitcast %"Expr<>"* %load_box to { %"Expr<>", i32 }*
  %rc = getelementptr inbounds { %"Expr<>", i32 }, { %"Expr<>", i32 }* %rc_release_box_cast, i32 0, i32 1
  %load_rc = load i32, i32* %rc
  %decrement = sub i32 %load_rc, 1
  store i32 %decrement, i32* %rc
  %1 = sext i32 %decrement to i64
  %print_rc = call {} @print(i64 %1)
  %rc_cmp = icmp eq i32 %decrement, 0
  br i1 %rc_cmp, label %rc_release_free, label %rc_release_ret

rc_release_free:                                  ; preds = %rc_release_start
  ret void

rc_release_ret:                                   ; preds = %rc_release_start
  ret void
}
