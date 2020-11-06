; ModuleID = 'main'
source_filename = "main"

%"List<>" = type { i16, { i64, %"List<>"* } }
%"Expr<>" = type { i16, { %"Expr<>"*, %"Expr<>"* } }

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

define i64 @"addr<List<>>"(%"List<>"* %0) {
addr_entry:
  %ptr_to_int = ptrtoint %"List<>"* %0 to i64
  ret i64 %ptr_to_int
}

define i64 @"addr<Expr<>>"(%"Expr<>"* %0) {
addr_entry:
  %ptr_to_int = ptrtoint %"Expr<>"* %0 to i64
  ret i64 %ptr_to_int
}

define %"Expr<>" @"Expr::Int<>"(i64 %0) {
basic_blockbb0:
  %ret = alloca %"Expr<>"
  %1 = alloca i64
  store i64 %0, i64* %1
  %discr_gep = getelementptr inbounds %"Expr<>", %"Expr<>"* %ret, i32 0, i32 0
  store i16 0, i16* %discr_gep
  %enum_gep = getelementptr inbounds %"Expr<>", %"Expr<>"* %ret, i32 0, i32 1
  %enum_ptr_cast = bitcast { %"Expr<>"*, %"Expr<>"* }* %enum_gep to { i64 }*
  %load = load i64, i64* %1
  %enum_content_gep = getelementptr inbounds { i64 }, { i64 }* %enum_ptr_cast, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load_ret = load %"Expr<>", %"Expr<>"* %ret
  ret %"Expr<>" %load_ret
}

define i64 @"eval<>"(%"Expr<>"* %0) {
basic_blockbb0:
  %ret = alloca i64
  %expr = alloca %"Expr<>"*
  store %"Expr<>"* %0, %"Expr<>"** %expr
  %tmp = alloca i1
  %tmp1 = alloca i16
  %tmp2 = alloca i1
  %i = alloca i64
  %tmp3 = alloca i1
  %tmp4 = alloca i16
  %tmp5 = alloca i1
  %l = alloca %"Expr<>"*
  %r = alloca %"Expr<>"*
  %tmp6 = alloca {}
  %tmp7 = alloca i64
  %tmp8 = alloca {}
  %tmp9 = alloca i64
  %tmp10 = alloca i64
  %tmp11 = alloca i64
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  store i1 true, i1* %tmp
  %load_deref = load %"Expr<>"*, %"Expr<>"** %expr
  %discr_gep = getelementptr inbounds %"Expr<>", %"Expr<>"* %load_deref, i32 0, i32 0
  %load_discr = load i16, i16* %discr_gep
  store i16 %load_discr, i16* %tmp1
  %load = load i16, i16* %tmp1
  %icmp_eq = icmp eq i16 0, %load
  store i1 %icmp_eq, i1* %tmp2
  %load12 = load i1, i1* %tmp2
  %load13 = load i1, i1* %tmp
  %and = and i1 %load12, %load13
  store i1 %and, i1* %tmp
  %load_deref14 = load %"Expr<>"*, %"Expr<>"** %expr
  %struct_gep = getelementptr inbounds %"Expr<>", %"Expr<>"* %load_deref14, i32 0, i32 1
  %lvalue_pointer_cast = bitcast { %"Expr<>"*, %"Expr<>"* }* %struct_gep to { i64 }*
  %struct_gep15 = getelementptr inbounds { i64 }, { i64 }* %lvalue_pointer_cast, i32 0, i32 0
  %load16 = load i64, i64* %struct_gep15
  store i64 %load16, i64* %i
  %load17 = load i1, i1* %tmp
  br i1 %load17, label %basic_blockbb2, label %basic_blockbb3

basic_blockbb2:                                   ; preds = %basic_blockbb1
  %load18 = load i64, i64* %i
  store i64 %load18, i64* %ret
  br label %basic_blockbb5

basic_blockbb3:                                   ; preds = %basic_blockbb1
  store i1 true, i1* %tmp3
  %load_deref19 = load %"Expr<>"*, %"Expr<>"** %expr
  %discr_gep20 = getelementptr inbounds %"Expr<>", %"Expr<>"* %load_deref19, i32 0, i32 0
  %load_discr21 = load i16, i16* %discr_gep20
  store i16 %load_discr21, i16* %tmp4
  %load22 = load i16, i16* %tmp4
  %icmp_eq23 = icmp eq i16 1, %load22
  store i1 %icmp_eq23, i1* %tmp5
  %load24 = load i1, i1* %tmp5
  %load25 = load i1, i1* %tmp3
  %and26 = and i1 %load24, %load25
  store i1 %and26, i1* %tmp3
  %load_deref27 = load %"Expr<>"*, %"Expr<>"** %expr
  %struct_gep28 = getelementptr inbounds %"Expr<>", %"Expr<>"* %load_deref27, i32 0, i32 1
  %struct_gep29 = getelementptr inbounds { %"Expr<>"*, %"Expr<>"* }, { %"Expr<>"*, %"Expr<>"* }* %struct_gep28, i32 0, i32 0
  %load30 = load %"Expr<>"*, %"Expr<>"** %struct_gep29
  store %"Expr<>"* %load30, %"Expr<>"** %l
  %load_deref31 = load %"Expr<>"*, %"Expr<>"** %expr
  %struct_gep32 = getelementptr inbounds %"Expr<>", %"Expr<>"* %load_deref31, i32 0, i32 1
  %struct_gep33 = getelementptr inbounds { %"Expr<>"*, %"Expr<>"* }, { %"Expr<>"*, %"Expr<>"* }* %struct_gep32, i32 0, i32 1
  %load34 = load %"Expr<>"*, %"Expr<>"** %struct_gep33
  store %"Expr<>"* %load34, %"Expr<>"** %r
  %load35 = load i1, i1* %tmp3
  br i1 %load35, label %basic_blockbb4, label %basic_blockbb6

basic_blockbb4:                                   ; preds = %basic_blockbb3
  %load36 = load %"Expr<>"*, %"Expr<>"** %l
  %fcall = call i64 @"addr<Expr<>>"(%"Expr<>"* %load36)
  store i64 %fcall, i64* %tmp7
  br label %basic_blockbb7

basic_blockbb5:                                   ; preds = %basic_blockbb12, %basic_blockbb2
  %load_ret = load i64, i64* %ret
  ret i64 %load_ret

basic_blockbb6:                                   ; preds = %basic_blockbb3
  call void @exit(i32 1)
  unreachable

basic_blockbb7:                                   ; preds = %basic_blockbb4
  %load37 = load i64, i64* %tmp7
  %fcall38 = call {} @print(i64 %load37)
  store {} %fcall38, {}* %tmp6
  br label %basic_blockbb8

basic_blockbb8:                                   ; preds = %basic_blockbb7
  %load39 = load %"Expr<>"*, %"Expr<>"** %r
  %fcall40 = call i64 @"addr<Expr<>>"(%"Expr<>"* %load39)
  store i64 %fcall40, i64* %tmp9
  br label %basic_blockbb9

basic_blockbb9:                                   ; preds = %basic_blockbb8
  %load41 = load i64, i64* %tmp9
  %fcall42 = call {} @print(i64 %load41)
  store {} %fcall42, {}* %tmp8
  br label %basic_blockbb10

basic_blockbb10:                                  ; preds = %basic_blockbb9
  %load43 = load %"Expr<>"*, %"Expr<>"** %l
  %fcall44 = call i64 @"eval<>"(%"Expr<>"* %load43)
  store i64 %fcall44, i64* %tmp10
  br label %basic_blockbb11

basic_blockbb11:                                  ; preds = %basic_blockbb10
  %load45 = load %"Expr<>"*, %"Expr<>"** %r
  %fcall46 = call i64 @"eval<>"(%"Expr<>"* %load45)
  store i64 %fcall46, i64* %tmp11
  br label %basic_blockbb12

basic_blockbb12:                                  ; preds = %basic_blockbb11
  %load47 = load i64, i64* %tmp10
  %load48 = load i64, i64* %tmp11
  %iadd = add i64 %load47, %load48
  store i64 %iadd, i64* %ret
  br label %basic_blockbb5
}

define %"Expr<>" @"Expr::Add<>"(%"Expr<>"* %0, %"Expr<>"* %1) {
basic_blockbb0:
  %ret = alloca %"Expr<>"
  %2 = alloca %"Expr<>"*
  store %"Expr<>"* %0, %"Expr<>"** %2
  %3 = alloca %"Expr<>"*
  store %"Expr<>"* %1, %"Expr<>"** %3
  %discr_gep = getelementptr inbounds %"Expr<>", %"Expr<>"* %ret, i32 0, i32 0
  store i16 1, i16* %discr_gep
  %enum_gep = getelementptr inbounds %"Expr<>", %"Expr<>"* %ret, i32 0, i32 1
  %load = load %"Expr<>"*, %"Expr<>"** %2
  %enum_content_gep = getelementptr inbounds { %"Expr<>"*, %"Expr<>"* }, { %"Expr<>"*, %"Expr<>"* }* %enum_gep, i32 0, i32 0
  store %"Expr<>"* %load, %"Expr<>"** %enum_content_gep
  %load1 = load %"Expr<>"*, %"Expr<>"** %3
  %enum_content_gep2 = getelementptr inbounds { %"Expr<>"*, %"Expr<>"* }, { %"Expr<>"*, %"Expr<>"* }* %enum_gep, i32 0, i32 1
  store %"Expr<>"* %load1, %"Expr<>"** %enum_content_gep2
  %load_ret = load %"Expr<>", %"Expr<>"* %ret
  ret %"Expr<>" %load_ret
}

define i64 @main() {
basic_blockbb0:
  %ret = alloca i64
  %tmp = alloca %"Expr<>"*
  %tmp1 = alloca %"Expr<>"
  %expr0 = alloca %"Expr<>"*
  %tmp2 = alloca {}
  %tmp3 = alloca i64
  %tmp4 = alloca %"Expr<>"*
  %tmp5 = alloca %"Expr<>"
  %expr1 = alloca %"Expr<>"*
  %tmp6 = alloca {}
  %tmp7 = alloca i64
  %tmp8 = alloca %"Expr<>"*
  %tmp9 = alloca %"Expr<>"
  %expr = alloca %"Expr<>"*
  %tmp10 = alloca {}
  %tmp11 = alloca i64
  %tmp12 = alloca i64
  %fcall = call %"Expr<>" @"Expr::Int<>"(i64 5)
  store %"Expr<>" %fcall, %"Expr<>"* %tmp1
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load = load %"Expr<>", %"Expr<>"* %tmp1
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ %"Expr<>", i32 }* getelementptr ({ %"Expr<>", i32 }, { %"Expr<>", i32 }* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to { %"Expr<>", i32 }*
  %rc_gep = getelementptr inbounds { %"Expr<>", i32 }, { %"Expr<>", i32 }* %box, i32 0, i32 1
  store i32 0, i32* %rc_gep
  %box_gep = getelementptr inbounds { %"Expr<>", i32 }, { %"Expr<>", i32 }* %box, i32 0, i32 0
  store %"Expr<>" %load, %"Expr<>"* %box_gep
  store %"Expr<>"* %box_gep, %"Expr<>"** %tmp
  %load13 = load %"Expr<>"*, %"Expr<>"** %tmp
  store %"Expr<>"* %load13, %"Expr<>"** %expr0
  %load14 = load %"Expr<>"*, %"Expr<>"** %expr0
  %fcall15 = call i64 @"addr<Expr<>>"(%"Expr<>"* %load14)
  store i64 %fcall15, i64* %tmp3
  br label %basic_blockbb2

basic_blockbb2:                                   ; preds = %basic_blockbb1
  %load16 = load i64, i64* %tmp3
  %fcall17 = call {} @print(i64 %load16)
  store {} %fcall17, {}* %tmp2
  br label %basic_blockbb3

basic_blockbb3:                                   ; preds = %basic_blockbb2
  %fcall18 = call %"Expr<>" @"Expr::Int<>"(i64 9)
  store %"Expr<>" %fcall18, %"Expr<>"* %tmp5
  br label %basic_blockbb4

basic_blockbb4:                                   ; preds = %basic_blockbb3
  %load19 = load %"Expr<>", %"Expr<>"* %tmp5
  %malloccall20 = tail call i8* @malloc(i32 ptrtoint ({ %"Expr<>", i32 }* getelementptr ({ %"Expr<>", i32 }, { %"Expr<>", i32 }* null, i32 1) to i32))
  %box21 = bitcast i8* %malloccall20 to { %"Expr<>", i32 }*
  %rc_gep22 = getelementptr inbounds { %"Expr<>", i32 }, { %"Expr<>", i32 }* %box21, i32 0, i32 1
  store i32 0, i32* %rc_gep22
  %box_gep23 = getelementptr inbounds { %"Expr<>", i32 }, { %"Expr<>", i32 }* %box21, i32 0, i32 0
  store %"Expr<>" %load19, %"Expr<>"* %box_gep23
  store %"Expr<>"* %box_gep23, %"Expr<>"** %tmp4
  %load24 = load %"Expr<>"*, %"Expr<>"** %tmp4
  store %"Expr<>"* %load24, %"Expr<>"** %expr1
  %load25 = load %"Expr<>"*, %"Expr<>"** %expr1
  %fcall26 = call i64 @"addr<Expr<>>"(%"Expr<>"* %load25)
  store i64 %fcall26, i64* %tmp7
  br label %basic_blockbb5

basic_blockbb5:                                   ; preds = %basic_blockbb4
  %load27 = load i64, i64* %tmp7
  %fcall28 = call {} @print(i64 %load27)
  store {} %fcall28, {}* %tmp6
  br label %basic_blockbb6

basic_blockbb6:                                   ; preds = %basic_blockbb5
  %load29 = load %"Expr<>"*, %"Expr<>"** %expr0
  %load30 = load %"Expr<>"*, %"Expr<>"** %expr1
  %fcall31 = call %"Expr<>" @"Expr::Add<>"(%"Expr<>"* %load29, %"Expr<>"* %load30)
  store %"Expr<>" %fcall31, %"Expr<>"* %tmp9
  br label %basic_blockbb7

basic_blockbb7:                                   ; preds = %basic_blockbb6
  %load32 = load %"Expr<>", %"Expr<>"* %tmp9
  %malloccall33 = tail call i8* @malloc(i32 ptrtoint ({ %"Expr<>", i32 }* getelementptr ({ %"Expr<>", i32 }, { %"Expr<>", i32 }* null, i32 1) to i32))
  %box34 = bitcast i8* %malloccall33 to { %"Expr<>", i32 }*
  %rc_gep35 = getelementptr inbounds { %"Expr<>", i32 }, { %"Expr<>", i32 }* %box34, i32 0, i32 1
  store i32 0, i32* %rc_gep35
  %box_gep36 = getelementptr inbounds { %"Expr<>", i32 }, { %"Expr<>", i32 }* %box34, i32 0, i32 0
  store %"Expr<>" %load32, %"Expr<>"* %box_gep36
  store %"Expr<>"* %box_gep36, %"Expr<>"** %tmp8
  %load37 = load %"Expr<>"*, %"Expr<>"** %tmp8
  store %"Expr<>"* %load37, %"Expr<>"** %expr
  %load38 = load %"Expr<>"*, %"Expr<>"** %expr
  %fcall39 = call i64 @"addr<Expr<>>"(%"Expr<>"* %load38)
  store i64 %fcall39, i64* %tmp11
  br label %basic_blockbb8

basic_blockbb8:                                   ; preds = %basic_blockbb7
  %load40 = load i64, i64* %tmp11
  %fcall41 = call {} @print(i64 %load40)
  store {} %fcall41, {}* %tmp10
  br label %basic_blockbb9

basic_blockbb9:                                   ; preds = %basic_blockbb8
  %load42 = load %"Expr<>"*, %"Expr<>"** %expr
  %fcall43 = call i64 @"eval<>"(%"Expr<>"* %load42)
  store i64 %fcall43, i64* %tmp12
  br label %basic_blockbb10

basic_blockbb10:                                  ; preds = %basic_blockbb9
  store i64 8, i64* %ret
  %load_ret = load i64, i64* %ret
  ret i64 %load_ret
}

define %"List<>"* @"last2<>"(%"List<>"* %0) {
basic_blockbb0:
  %ret = alloca %"List<>"*
  %list = alloca %"List<>"*
  store %"List<>"* %0, %"List<>"** %list
  %tmp = alloca i1
  %tmp1 = alloca i16
  %tmp2 = alloca i1
  %i = alloca i64
  %l = alloca %"List<>"*
  %tmp3 = alloca {}
  %tmp4 = alloca i64
  %tmp5 = alloca i1
  %tmp6 = alloca i16
  %tmp7 = alloca i1
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  store i1 true, i1* %tmp
  %load_deref = load %"List<>"*, %"List<>"** %list
  %discr_gep = getelementptr inbounds %"List<>", %"List<>"* %load_deref, i32 0, i32 0
  %load_discr = load i16, i16* %discr_gep
  store i16 %load_discr, i16* %tmp1
  %load = load i16, i16* %tmp1
  %icmp_eq = icmp eq i16 0, %load
  store i1 %icmp_eq, i1* %tmp2
  %load8 = load i1, i1* %tmp2
  %load9 = load i1, i1* %tmp
  %and = and i1 %load8, %load9
  store i1 %and, i1* %tmp
  %load_deref10 = load %"List<>"*, %"List<>"** %list
  %struct_gep = getelementptr inbounds %"List<>", %"List<>"* %load_deref10, i32 0, i32 1
  %struct_gep11 = getelementptr inbounds { i64, %"List<>"* }, { i64, %"List<>"* }* %struct_gep, i32 0, i32 0
  %load12 = load i64, i64* %struct_gep11
  store i64 %load12, i64* %i
  %load_deref13 = load %"List<>"*, %"List<>"** %list
  %struct_gep14 = getelementptr inbounds %"List<>", %"List<>"* %load_deref13, i32 0, i32 1
  %struct_gep15 = getelementptr inbounds { i64, %"List<>"* }, { i64, %"List<>"* }* %struct_gep14, i32 0, i32 1
  %load16 = load %"List<>"*, %"List<>"** %struct_gep15
  store %"List<>"* %load16, %"List<>"** %l
  %load17 = load i1, i1* %tmp
  br i1 %load17, label %basic_blockbb2, label %basic_blockbb3

basic_blockbb2:                                   ; preds = %basic_blockbb1
  %load18 = load %"List<>"*, %"List<>"** %l
  %fcall = call i64 @"addr<List<>>"(%"List<>"* %load18)
  store i64 %fcall, i64* %tmp4
  br label %basic_blockbb6

basic_blockbb3:                                   ; preds = %basic_blockbb1
  store i1 true, i1* %tmp5
  %load_deref19 = load %"List<>"*, %"List<>"** %list
  %discr_gep20 = getelementptr inbounds %"List<>", %"List<>"* %load_deref19, i32 0, i32 0
  %load_discr21 = load i16, i16* %discr_gep20
  store i16 %load_discr21, i16* %tmp6
  %load22 = load i16, i16* %tmp6
  %icmp_eq23 = icmp eq i16 1, %load22
  store i1 %icmp_eq23, i1* %tmp7
  %load24 = load i1, i1* %tmp7
  %load25 = load i1, i1* %tmp5
  %and26 = and i1 %load24, %load25
  store i1 %and26, i1* %tmp5
  %load27 = load i1, i1* %tmp5
  br i1 %load27, label %basic_blockbb4, label %basic_blockbb9

basic_blockbb4:                                   ; preds = %basic_blockbb3
  %load28 = load %"List<>"*, %"List<>"** %list
  store %"List<>"* %load28, %"List<>"** %ret
  br label %basic_blockbb5

basic_blockbb5:                                   ; preds = %basic_blockbb8, %basic_blockbb4
  %load_ret = load %"List<>"*, %"List<>"** %ret
  ret %"List<>"* %load_ret

basic_blockbb6:                                   ; preds = %basic_blockbb2
  %load29 = load i64, i64* %tmp4
  %fcall30 = call {} @print(i64 %load29)
  store {} %fcall30, {}* %tmp3
  br label %basic_blockbb7

basic_blockbb7:                                   ; preds = %basic_blockbb6
  %load31 = load %"List<>"*, %"List<>"** %l
  %fcall32 = call %"List<>"* @"last2<>"(%"List<>"* %load31)
  store %"List<>"* %fcall32, %"List<>"** %ret
  br label %basic_blockbb8

basic_blockbb8:                                   ; preds = %basic_blockbb7
  br label %basic_blockbb5

basic_blockbb9:                                   ; preds = %basic_blockbb3
  call void @exit(i32 1)
  unreachable
}

declare noalias i8* @malloc(i32)
