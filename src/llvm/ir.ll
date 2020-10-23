; ModuleID = 'main'
source_filename = "main"

%"Node<>" = type { i64, %"NodeOption<>" }
%"NodeOption<>" = type { i16, { %"Node<>"* } }

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
  %ret = alloca i64
  %tmp = alloca %"Node<>"
  %tmp1 = alloca %"NodeOption<>"
  %node = alloca %"Node<>"
  %tmp2 = alloca %"Node<>"
  %tmp3 = alloca %"NodeOption<>"
  %tmp4 = alloca %"Node<>"*
  %head = alloca %"Node<>"
  %tmp5 = alloca i1
  %tmp6 = alloca i16
  %tmp7 = alloca i1
  %n = alloca %"Node<>"*
  %tmp8 = alloca i1
  %tmp9 = alloca i16
  %tmp10 = alloca i1
  %discr_gep = getelementptr inbounds %"NodeOption<>", %"NodeOption<>"* %tmp1, i32 0, i32 0
  store i16 0, i16* %discr_gep
  %enum_gep = getelementptr inbounds %"NodeOption<>", %"NodeOption<>"* %tmp1, i32 0, i32 1
  %enum_ptr_cast = bitcast { %"Node<>"* }* %enum_gep to {}*
  %struct_gep = getelementptr inbounds %"Node<>", %"Node<>"* %tmp, i32 0, i32 0
  store i64 9, i64* %struct_gep
  %load = load %"NodeOption<>", %"NodeOption<>"* %tmp1
  %struct_gep11 = getelementptr inbounds %"Node<>", %"Node<>"* %tmp, i32 0, i32 1
  store %"NodeOption<>" %load, %"NodeOption<>"* %struct_gep11
  %load12 = load %"Node<>", %"Node<>"* %tmp
  store %"Node<>" %load12, %"Node<>"* %node
  %load13 = load %"Node<>", %"Node<>"* %node
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ %"Node<>", i32 }* getelementptr ({ %"Node<>", i32 }, { %"Node<>", i32 }* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to { %"Node<>", i32 }*
  %rc_gep = getelementptr inbounds { %"Node<>", i32 }, { %"Node<>", i32 }* %box, i32 0, i32 1
  store i32 0, i32* %rc_gep
  %box_gep = getelementptr inbounds { %"Node<>", i32 }, { %"Node<>", i32 }* %box, i32 0, i32 0
  store %"Node<>" %load13, %"Node<>"* %box_gep
  store %"Node<>"* %box_gep, %"Node<>"** %tmp4
  %load14 = load %"Node<>"*, %"Node<>"** %tmp4
  %fcall = call %"NodeOption<>" @"NodeOption::Some<>"(%"Node<>"* %load14)
  store %"NodeOption<>" %fcall, %"NodeOption<>"* %tmp3
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %struct_gep15 = getelementptr inbounds %"Node<>", %"Node<>"* %tmp2, i32 0, i32 0
  store i64 4, i64* %struct_gep15
  %load16 = load %"NodeOption<>", %"NodeOption<>"* %tmp3
  %struct_gep17 = getelementptr inbounds %"Node<>", %"Node<>"* %tmp2, i32 0, i32 1
  store %"NodeOption<>" %load16, %"NodeOption<>"* %struct_gep17
  %load18 = load %"Node<>", %"Node<>"* %tmp2
  store %"Node<>" %load18, %"Node<>"* %head
  br label %basic_blockbb2

basic_blockbb2:                                   ; preds = %basic_blockbb1
  store i1 true, i1* %tmp5
  %struct_gep19 = getelementptr inbounds %"Node<>", %"Node<>"* %head, i32 0, i32 1
  %discr_gep20 = getelementptr inbounds %"NodeOption<>", %"NodeOption<>"* %struct_gep19, i32 0, i32 0
  %load_discr = load i16, i16* %discr_gep20
  store i16 %load_discr, i16* %tmp6
  %load21 = load i16, i16* %tmp6
  %extend_discr = zext i16 %load21 to i64
  %icmp_eq = icmp eq i64 1, %extend_discr
  store i1 %icmp_eq, i1* %tmp7
  %load22 = load i1, i1* %tmp7
  %load23 = load i1, i1* %tmp5
  %and = and i1 %load22, %load23
  store i1 %and, i1* %tmp5
  %struct_gep24 = getelementptr inbounds %"Node<>", %"Node<>"* %head, i32 0, i32 1
  %struct_gep25 = getelementptr inbounds %"NodeOption<>", %"NodeOption<>"* %struct_gep24, i32 0, i32 1
  %struct_gep26 = getelementptr inbounds { %"Node<>"* }, { %"Node<>"* }* %struct_gep25, i32 0, i32 0
  %load27 = load %"Node<>"*, %"Node<>"** %struct_gep26
  store %"Node<>"* %load27, %"Node<>"** %n
  %load28 = load i1, i1* %tmp5
  br i1 %load28, label %basic_blockbb3, label %basic_blockbb4

basic_blockbb3:                                   ; preds = %basic_blockbb2
  %load_deref = load %"Node<>"*, %"Node<>"** %n
  %struct_gep29 = getelementptr inbounds %"Node<>", %"Node<>"* %load_deref, i32 0, i32 0
  %load30 = load i64, i64* %struct_gep29
  store i64 %load30, i64* %ret
  br label %basic_blockbb6

basic_blockbb4:                                   ; preds = %basic_blockbb2
  store i1 true, i1* %tmp8
  %struct_gep31 = getelementptr inbounds %"Node<>", %"Node<>"* %head, i32 0, i32 1
  %discr_gep32 = getelementptr inbounds %"NodeOption<>", %"NodeOption<>"* %struct_gep31, i32 0, i32 0
  %load_discr33 = load i16, i16* %discr_gep32
  store i16 %load_discr33, i16* %tmp9
  %load34 = load i16, i16* %tmp9
  %extend_discr35 = zext i16 %load34 to i64
  %icmp_eq36 = icmp eq i64 0, %extend_discr35
  store i1 %icmp_eq36, i1* %tmp10
  %load37 = load i1, i1* %tmp10
  %load38 = load i1, i1* %tmp8
  %and39 = and i1 %load37, %load38
  store i1 %and39, i1* %tmp8
  %load40 = load i1, i1* %tmp8
  br i1 %load40, label %basic_blockbb5, label %basic_blockbb7

basic_blockbb5:                                   ; preds = %basic_blockbb4
  store i64 0, i64* %ret
  br label %basic_blockbb6

basic_blockbb6:                                   ; preds = %basic_blockbb5, %basic_blockbb3
  %load_ret = load i64, i64* %ret
  ret i64 %load_ret

basic_blockbb7:                                   ; preds = %basic_blockbb4
  call void @exit(i32 1)
  unreachable
}

define %"NodeOption<>" @"NodeOption::Some<>"(%"Node<>"* %0) {
basic_blockbb0:
  %ret = alloca %"NodeOption<>"
  %1 = alloca %"Node<>"*
  store %"Node<>"* %0, %"Node<>"** %1
  %discr_gep = getelementptr inbounds %"NodeOption<>", %"NodeOption<>"* %ret, i32 0, i32 0
  store i16 1, i16* %discr_gep
  %enum_gep = getelementptr inbounds %"NodeOption<>", %"NodeOption<>"* %ret, i32 0, i32 1
  %load = load %"Node<>"*, %"Node<>"** %1
  %enum_content_gep = getelementptr inbounds { %"Node<>"* }, { %"Node<>"* }* %enum_gep, i32 0, i32 0
  store %"Node<>"* %load, %"Node<>"** %enum_content_gep
  %load_ret = load %"NodeOption<>", %"NodeOption<>"* %ret
  ret %"NodeOption<>" %load_ret
}

declare noalias i8* @malloc(i32)
