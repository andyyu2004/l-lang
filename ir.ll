; ModuleID = 'main'
source_filename = "main"

%"S<>" = type { i64 }
%"Option<int>" = type { i16, { i64 } }

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

define %"S<>" @"new<>"() {
basic_blockbb0:
  %ret = alloca %"S<>"
  %struct_gep = getelementptr inbounds %"S<>", %"S<>"* %ret, i32 0, i32 0
  store i64 5, i64* %struct_gep
  %load_ret = load %"S<>", %"S<>"* %ret
  ret %"S<>" %load_ret
}

define i64 @main() {
basic_blockbb0:
  %ret = alloca i64
  %tmp = alloca %"S<>"
  %tmp1 = alloca %"Option<int>"
  %k = alloca %"Option<int>"
  %tmp2 = alloca i64
  %tmp3 = alloca i1
  %tmp4 = alloca i16
  %tmp5 = alloca i1
  %x = alloca i64
  %tmp6 = alloca i1
  %struct_gep = getelementptr inbounds %"S<>", %"S<>"* %tmp, i32 0, i32 0
  store i64 9, i64* %struct_gep
  %fcall = call %"Option<int>" @"Option::Some<int>"(i64 5)
  store %"Option<int>" %fcall, %"Option<int>"* %tmp1
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load = load %"Option<int>", %"Option<int>"* %tmp1
  store %"Option<int>" %load, %"Option<int>"* %k
  br label %basic_blockbb2

basic_blockbb2:                                   ; preds = %basic_blockbb1
  store i1 true, i1* %tmp3
  %discr_gep = getelementptr inbounds %"Option<int>", %"Option<int>"* %k, i32 0, i32 0
  %load_discr = load i16, i16* %discr_gep
  store i16 %load_discr, i16* %tmp4
  %load7 = load i16, i16* %tmp4
  %extend_discr = zext i16 %load7 to i64
  %icmp_eq = icmp eq i64 0, %extend_discr
  store i1 %icmp_eq, i1* %tmp5
  %load8 = load i1, i1* %tmp5
  %load9 = load i1, i1* %tmp3
  %and = and i1 %load8, %load9
  store i1 %and, i1* %tmp3
  %struct_gep10 = getelementptr inbounds %"Option<int>", %"Option<int>"* %k, i32 0, i32 1
  %struct_gep11 = getelementptr inbounds { i64 }, { i64 }* %struct_gep10, i32 0, i32 0
  %load12 = load i64, i64* %struct_gep11
  store i64 %load12, i64* %x
  %load13 = load i1, i1* %tmp3
  br i1 %load13, label %basic_blockbb3, label %basic_blockbb4

basic_blockbb3:                                   ; preds = %basic_blockbb2
  %load14 = load i64, i64* %x
  store i64 %load14, i64* %tmp2
  br label %basic_blockbb6

basic_blockbb4:                                   ; preds = %basic_blockbb2
  store i1 true, i1* %tmp6
  %load15 = load i1, i1* %tmp6
  br i1 %load15, label %basic_blockbb5, label %basic_blockbb7

basic_blockbb5:                                   ; preds = %basic_blockbb4
  %load16 = load i64, i64* %x
  store i64 %load16, i64* %tmp2
  br label %basic_blockbb6

basic_blockbb6:                                   ; preds = %basic_blockbb5, %basic_blockbb3
  store i64 9, i64* %ret
  %load_ret = load i64, i64* %ret
  ret i64 %load_ret

basic_blockbb7:                                   ; preds = %basic_blockbb4
  call void @exit(i32 1)
  unreachable
}

define %"Option<int>" @"Option::Some<int>"(i64 %0) {
basic_blockbb0:
  %ret = alloca %"Option<int>"
  %1 = alloca i64
  store i64 %0, i64* %1
  %discr_gep = getelementptr inbounds %"Option<int>", %"Option<int>"* %ret, i32 0, i32 0
  store i16 0, i16* %discr_gep
  %enum_gep = getelementptr inbounds %"Option<int>", %"Option<int>"* %ret, i32 0, i32 1
  %load = load i64, i64* %1
  %enum_content_gep = getelementptr inbounds { i64 }, { i64 }* %enum_gep, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load_ret = load %"Option<int>", %"Option<int>"* %ret
  ret %"Option<int>" %load_ret
}
