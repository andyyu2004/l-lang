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

define {} @"mutate<>"(i64* %0) {
basic_blockbb0:
  %ret = alloca {}
  %ptr = alloca i64*
  store i64* %0, i64** %ptr
  %load_deref = load i64*, i64** %ptr
  store i64 99, i64* %load_deref
  store {} undef, {}* %ret
  %load_ret = load {}, {}* %ret
  ret {} %load_ret
}

define i64 @main() {
basic_blockbb0:
  %ret = alloca i64
  %tmp = alloca i64*
  %ptr = alloca i64*
  %tmp1 = alloca {}
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ i64, i32 }* getelementptr ({ i64, i32 }, { i64, i32 }* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to { i64, i32 }*
  %rc_gep = getelementptr inbounds { i64, i32 }, { i64, i32 }* %box, i32 0, i32 1
  store i32 0, i32* %rc_gep
  %box_gep = getelementptr inbounds { i64, i32 }, { i64, i32 }* %box, i32 0, i32 0
  store i64 5, i64* %box_gep
  store i64* %box_gep, i64** %tmp
  %load = load i64*, i64** %tmp
  store i64* %load, i64** %ptr
  %load2 = load i64*, i64** %ptr
  %fcall = call {} @"mutate<>"(i64* %load2)
  store {} %fcall, {}* %tmp1
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load_deref = load i64*, i64** %ptr
  %load3 = load i64, i64* %load_deref
  store i64 %load3, i64* %ret
  %load_ret = load i64, i64* %ret
  ret i64 %load_ret
}

declare noalias i8* @malloc(i32)
