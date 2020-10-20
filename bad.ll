; ModuleID = 'main'
source_filename = "main"

%"List<>" = type { i64, { i64, %"List<>"* } }

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

define %"List<>"* @"last2<>"(%"List<>"* %0) {
basic_blockbb0:
  %retvar = alloca %"List<>"*
  %list = alloca %"List<>"*
  store %"List<>"* %0, %"List<>"** %list
  %tmp = alloca i1
  %tmp1 = alloca i64
  %tmp2 = alloca i1
  %i = alloca i64
  %l = alloca %"List<>"*
  %tmp3 = alloca i1
  %tmp4 = alloca i64
  %tmp5 = alloca i1
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  store i1 true, i1* %tmp
  %load_deref = load %"List<>"*, %"List<>"** %list
  %discr_gep = getelementptr inbounds %"List<>", %"List<>"* %load_deref, i32 0, i32 0
  %load_discr = load i64, i64* %discr_gep
  store i64 %load_discr, i64* %tmp1
  %load = load i64, i64* %tmp1
  %icmp_eq = icmp eq i64 0, %load
  store i1 %icmp_eq, i1* %tmp2
  %load6 = load i1, i1* %tmp2
  %load7 = load i1, i1* %tmp
  %and = and i1 %load6, %load7
  store i1 %and, i1* %tmp
  %load_deref8 = load %"List<>"*, %"List<>"** %list
  %struct_gep = getelementptr inbounds %"List<>", %"List<>"* %load_deref8, i32 0, i32 1
  %struct_gep9 = getelementptr inbounds { i64, %"List<>"* }, { i64, %"List<>"* }* %struct_gep, i32 0, i32 0
  %load10 = load i64, i64* %struct_gep9
  store i64 %load10, i64* %i
  %load_deref11 = load %"List<>"*, %"List<>"** %list
  %struct_gep12 = getelementptr inbounds %"List<>", %"List<>"* %load_deref11, i32 0, i32 1
  %struct_gep13 = getelementptr inbounds { i64, %"List<>"* }, { i64, %"List<>"* }* %struct_gep12, i32 0, i32 1
  %load14 = load %"List<>"*, %"List<>"** %struct_gep13
  store %"List<>"* %load14, %"List<>"** %l
  ;call void @"rc_retain<List<>>"(%"List<>"** %l)
  %load15 = load i1, i1* %tmp
  br i1 %load15, label %basic_blockbb2, label %basic_blockbb3

basic_blockbb2:                                   ; preds = %basic_blockbb1
  %load16 = load %"List<>"*, %"List<>"** %l
  %fcall = call %"List<>"* @"last2<>.1"(%"List<>"* %load16)
  store %"List<>"* %fcall, %"List<>"** %retvar
  br label %basic_blockbb6

basic_blockbb3:                                   ; preds = %basic_blockbb1
  store i1 true, i1* %tmp3
  %load_deref17 = load %"List<>"*, %"List<>"** %list
  %discr_gep18 = getelementptr inbounds %"List<>", %"List<>"* %load_deref17, i32 0, i32 0
  %load_discr19 = load i64, i64* %discr_gep18
  store i64 %load_discr19, i64* %tmp4
  %load20 = load i64, i64* %tmp4
  %icmp_eq21 = icmp eq i64 1, %load20
  store i1 %icmp_eq21, i1* %tmp5
  %load22 = load i1, i1* %tmp5
  %load23 = load i1, i1* %tmp3
  %and24 = and i1 %load22, %load23
  store i1 %and24, i1* %tmp3
  %load25 = load i1, i1* %tmp3
  br i1 %load25, label %basic_blockbb4, label %basic_blockbb7

basic_blockbb4:                                   ; preds = %basic_blockbb3
  %load26 = load %"List<>"*, %"List<>"** %list
  store %"List<>"* %load26, %"List<>"** %retvar
  ;call void @"rc_retain<List<>>"(%"List<>"** %retvar)
  br label %basic_blockbb5

basic_blockbb5:                                   ; preds = %basic_blockbb6, %basic_blockbb4
  %load_ret = load %"List<>"*, %"List<>"** %retvar
  ret %"List<>"* %load_ret

basic_blockbb6:                                   ; preds = %basic_blockbb2
  br label %basic_blockbb5

basic_blockbb7:                                   ; preds = %basic_blockbb3
  call void @exit(i32 1)
  unreachable
}

define i64 @main() {
basic_blockbb0:
  %retvar = alloca i64
  %tmp = alloca %"List<>"*
  %tmp1 = alloca %"List<>"
  %tail = alloca %"List<>"*
  %tmp2 = alloca %"List<>"*
  %tmp3 = alloca %"List<>"
  %head = alloca %"List<>"*
  %tmp4 = alloca %"List<>"*
  %discr_gep = getelementptr inbounds %"List<>", %"List<>"* %tmp1, i32 0, i32 0
  store i64 1, i64* %discr_gep
  %enum_gep = getelementptr inbounds %"List<>", %"List<>"* %tmp1, i32 0, i32 1
  %enum_ptr_cast = bitcast { i64, %"List<>"* }* %enum_gep to {}*
  %load = load %"List<>", %"List<>"* %tmp1
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ %"List<>", i32 }* getelementptr ({ %"List<>", i32 }, { %"List<>", i32 }* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to { %"List<>", i32 }*
  %alloc_str = alloca [12 x i8]
  store [12 x i8] c"malloc box \00", [12 x i8]* %alloc_str
  %cast_str = bitcast [12 x i8]* %alloc_str to i8*
  %print_str = call i32 (i8*, ...) @printf(i8* %cast_str)
  %cast_malloc_ptr = bitcast { %"List<>", i32 }* %box to i8*
  %print_malloc_addr = call {} @print_addr(i8* %cast_malloc_ptr)
  %rc_gep = getelementptr inbounds { %"List<>", i32 }, { %"List<>", i32 }* %box, i32 0, i32 1
  store i32 0, i32* %rc_gep
  %box_gep = getelementptr inbounds { %"List<>", i32 }, { %"List<>", i32 }* %box, i32 0, i32 0
  store %"List<>" %load, %"List<>"* %box_gep
  store %"List<>"* %box_gep, %"List<>"** %tmp
  ;call void @"rc_retain<List<>>"(%"List<>"** %tmp)
  %load5 = load %"List<>"*, %"List<>"** %tmp
  store %"List<>"* %load5, %"List<>"** %tail
  ;call void @"rc_retain<List<>>"(%"List<>"** %tail)
  %load6 = load %"List<>"*, %"List<>"** %tail
  %fcall = call %"List<>" @"List::Next<>"(i64 4, %"List<>"* %load6)
  store %"List<>" %fcall, %"List<>"* %tmp3
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load7 = load %"List<>", %"List<>"* %tmp3
  %malloccall8 = tail call i8* @malloc(i32 ptrtoint ({ %"List<>", i32 }* getelementptr ({ %"List<>", i32 }, { %"List<>", i32 }* null, i32 1) to i32))
  %box9 = bitcast i8* %malloccall8 to { %"List<>", i32 }*
  %alloc_str10 = alloca [12 x i8]
  store [12 x i8] c"malloc box \00", [12 x i8]* %alloc_str10
  %cast_str11 = bitcast [12 x i8]* %alloc_str10 to i8*
  %print_str12 = call i32 (i8*, ...) @printf(i8* %cast_str11)
  %cast_malloc_ptr13 = bitcast { %"List<>", i32 }* %box9 to i8*
  %print_malloc_addr14 = call {} @print_addr(i8* %cast_malloc_ptr13)
  %rc_gep15 = getelementptr inbounds { %"List<>", i32 }, { %"List<>", i32 }* %box9, i32 0, i32 1
  store i32 0, i32* %rc_gep15
  %box_gep16 = getelementptr inbounds { %"List<>", i32 }, { %"List<>", i32 }* %box9, i32 0, i32 0
  store %"List<>" %load7, %"List<>"* %box_gep16
  store %"List<>"* %box_gep16, %"List<>"** %tmp2
  ;call void @"rc_retain<List<>>"(%"List<>"** %tmp2)
  %load17 = load %"List<>"*, %"List<>"** %tmp2
  store %"List<>"* %load17, %"List<>"** %head
  ;call void @"rc_retain<List<>>"(%"List<>"** %head)
  %load18 = load %"List<>"*, %"List<>"** %head
  %fcall19 = call %"List<>"* @"last2<>.1"(%"List<>"* %load18)
  store %"List<>"* %fcall19, %"List<>"** %tmp4
  br label %basic_blockbb2

basic_blockbb2:                                   ; preds = %basic_blockbb1
  store i64 8, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

define %"List<>"* @"last2<>.1"(%"List<>"* %0) {
basic_blockbb0:
  %retvar = alloca %"List<>"*
  %list = alloca %"List<>"*
  store %"List<>"* %0, %"List<>"** %list
  %tmp = alloca i1
  %tmp1 = alloca i64
  %tmp2 = alloca i1
  %i = alloca i64
  %l = alloca %"List<>"*
  %tmp3 = alloca i1
  %tmp4 = alloca i64
  %tmp5 = alloca i1
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  store i1 true, i1* %tmp
  %load_deref = load %"List<>"*, %"List<>"** %list
  %discr_gep = getelementptr inbounds %"List<>", %"List<>"* %load_deref, i32 0, i32 0
  %load_discr = load i64, i64* %discr_gep
  store i64 %load_discr, i64* %tmp1
  %load = load i64, i64* %tmp1
  %icmp_eq = icmp eq i64 0, %load
  store i1 %icmp_eq, i1* %tmp2
  %load6 = load i1, i1* %tmp2
  %load7 = load i1, i1* %tmp
  %and = and i1 %load6, %load7
  store i1 %and, i1* %tmp
  %load_deref8 = load %"List<>"*, %"List<>"** %list
  %struct_gep = getelementptr inbounds %"List<>", %"List<>"* %load_deref8, i32 0, i32 1
  %struct_gep9 = getelementptr inbounds { i64, %"List<>"* }, { i64, %"List<>"* }* %struct_gep, i32 0, i32 0
  %load10 = load i64, i64* %struct_gep9
  store i64 %load10, i64* %i
  %load_deref11 = load %"List<>"*, %"List<>"** %list
  %struct_gep12 = getelementptr inbounds %"List<>", %"List<>"* %load_deref11, i32 0, i32 1
  %struct_gep13 = getelementptr inbounds { i64, %"List<>"* }, { i64, %"List<>"* }* %struct_gep12, i32 0, i32 1
  %load14 = load %"List<>"*, %"List<>"** %struct_gep13
  store %"List<>"* %load14, %"List<>"** %l
  call void @"rc_retain<List<>>"(%"List<>"** %l)
  %load15 = load i1, i1* %tmp
  br i1 %load15, label %basic_blockbb2, label %basic_blockbb3

basic_blockbb2:                                   ; preds = %basic_blockbb1
  %load16 = load %"List<>"*, %"List<>"** %l
  %fcall = call %"List<>"* @"last2<>.1"(%"List<>"* %load16)
  store %"List<>"* %fcall, %"List<>"** %retvar
  br label %basic_blockbb6

basic_blockbb3:                                   ; preds = %basic_blockbb1
  store i1 true, i1* %tmp3
  %load_deref17 = load %"List<>"*, %"List<>"** %list
  %discr_gep18 = getelementptr inbounds %"List<>", %"List<>"* %load_deref17, i32 0, i32 0
  %load_discr19 = load i64, i64* %discr_gep18
  store i64 %load_discr19, i64* %tmp4
  %load20 = load i64, i64* %tmp4
  %icmp_eq21 = icmp eq i64 1, %load20
  store i1 %icmp_eq21, i1* %tmp5
  %load22 = load i1, i1* %tmp5
  %load23 = load i1, i1* %tmp3
  %and24 = and i1 %load22, %load23
  store i1 %and24, i1* %tmp3
  %load25 = load i1, i1* %tmp3
  br i1 %load25, label %basic_blockbb4, label %basic_blockbb7

basic_blockbb4:                                   ; preds = %basic_blockbb3
  %load26 = load %"List<>"*, %"List<>"** %list
  store %"List<>"* %load26, %"List<>"** %retvar
  ;call void @"rc_retain<List<>>"(%"List<>"** %retvar)
  br label %basic_blockbb5

basic_blockbb5:                                   ; preds = %basic_blockbb6, %basic_blockbb4
  %load_ret = load %"List<>"*, %"List<>"** %retvar
  ret %"List<>"* %load_ret

basic_blockbb6:                                   ; preds = %basic_blockbb2
  br label %basic_blockbb5

basic_blockbb7:                                   ; preds = %basic_blockbb3
  call void @exit(i32 1)
  unreachable
}

define %"List<>" @"List::Next<>"(i64 %0, %"List<>"* %1) {
basic_blockbb0:
  %retvar = alloca %"List<>"
  %2 = alloca i64
  store i64 %0, i64* %2
  %3 = alloca %"List<>"*
  store %"List<>"* %1, %"List<>"** %3
  %discr_gep = getelementptr inbounds %"List<>", %"List<>"* %retvar, i32 0, i32 0
  store i64 0, i64* %discr_gep
  %enum_gep = getelementptr inbounds %"List<>", %"List<>"* %retvar, i32 0, i32 1
  %load = load i64, i64* %2
  %enum_content_gep = getelementptr inbounds { i64, %"List<>"* }, { i64, %"List<>"* }* %enum_gep, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load1 = load %"List<>"*, %"List<>"** %3
  %enum_content_gep2 = getelementptr inbounds { i64, %"List<>"* }, { i64, %"List<>"* }* %enum_gep, i32 0, i32 1
  store %"List<>"* %load1, %"List<>"** %enum_content_gep2
  %load_ret = load %"List<>", %"List<>"* %retvar
  ret %"List<>" %load_ret
}

declare noalias i8* @malloc(i32)

define void @"rc_retain<List<>>"(%"List<>"** %0) {
rc_retain_start:
  %alloc_str = alloca [17 x i8]
  store [17 x i8] c"rc_retain_count\0A\00", [17 x i8]* %alloc_str
  %cast_str = bitcast [17 x i8]* %alloc_str to i8*
  %print_str = call i32 (i8*, ...) @printf(i8* %cast_str)
  %load_box = load %"List<>"*, %"List<>"** %0
  %cast_malloc_ptr = bitcast %"List<>"* %load_box to i8*
  %print_malloc_addr = call {} @print_addr(i8* %cast_malloc_ptr)
  %rc_retain_box_cast = bitcast %"List<>"* %load_box to { %"List<>", i32 }*
  %rc = getelementptr inbounds { %"List<>", i32 }, { %"List<>", i32 }* %rc_retain_box_cast, i32 0, i32 1
  %load_rc = load i32, i32* %rc
  %increment_rc = add i32 %load_rc, 1
  store i32 %increment_rc, i32* %rc
  %i64rc = sext i32 %increment_rc to i64
  %rc_retain_count = call {} @print(i64 %i64rc)
  ret void
}
