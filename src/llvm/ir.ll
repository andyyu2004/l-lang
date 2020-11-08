; ModuleID = 'main'
source_filename = "main"

%"S<>" = type { i64, { i64, i1, i64 } }

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
  %tmp = alloca %"S<>"
  %tmp1 = alloca { i64, i1, i64 }
  %struct_gep = getelementptr inbounds { i64, i1, i64 }, { i64, i1, i64 }* %tmp1, i32 0, i32 0
  store i64 4, i64* %struct_gep
  %struct_gep2 = getelementptr inbounds { i64, i1, i64 }, { i64, i1, i64 }* %tmp1, i32 0, i32 1
  store i1 false, i1* %struct_gep2
  %struct_gep3 = getelementptr inbounds { i64, i1, i64 }, { i64, i1, i64 }* %tmp1, i32 0, i32 2
  store i64 9, i64* %struct_gep3
  %struct_gep4 = getelementptr inbounds %"S<>", %"S<>"* %tmp, i32 0, i32 0
  store i64 5, i64* %struct_gep4
  %load = load { i64, i1, i64 }, { i64, i1, i64 }* %tmp1
  %struct_gep5 = getelementptr inbounds %"S<>", %"S<>"* %tmp, i32 0, i32 1
  store { i64, i1, i64 } %load, { i64, i1, i64 }* %struct_gep5
  %struct_gep6 = getelementptr inbounds %"S<>", %"S<>"* %tmp, i32 0, i32 1
  %struct_gep7 = getelementptr inbounds { i64, i1, i64 }, { i64, i1, i64 }* %struct_gep6, i32 0, i32 2
  %load8 = load i64, i64* %struct_gep7
  store i64 %load8, i64* %ret
  %load_ret = load i64, i64* %ret
  ret i64 %load_ret
}
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          t_gep24 = getelementptr inbounds %"Node<>", %"Node<>"* %head, i32 0, i32 1
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
  %icmp_eq35 = icmp eq i16 0, %load34
  store i1 %icmp_eq35, i1* %tmp10
  %load36 = load i1, i1* %tmp10
  %load37 = load i1, i1* %tmp8
  %and38 = and i1 %load36, %load37
  store i1 %and38, i1* %tmp8
  %load39 = load i1, i1* %tmp8
  br i1 %load39, label %basic_blockbb5, label %basic_blockbb7

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
