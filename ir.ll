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

define i64 @main() {
basic_blockbb0:
  %ret = alloca i64
  %tmp = alloca {}
  %tmp1 = alloca i64
  %fcall = call i64 @"fuck<>"()
  store i64 %fcall, i64* %tmp1
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load = load i64, i64* %tmp1
  %fcall2 = call {} @print(i64 %load)
  store {} %fcall2, {}* %tmp
  br label %basic_blockbb2

basic_blockbb2:                                   ; preds = %basic_blockbb1
  store i64 9, i64* %ret
  %load_ret = load i64, i64* %ret
  ret i64 %load_ret
}

define i64 @"fuck<>"() {
basic_blockbb0:
  %ret = alloca i64
  store i64 5, i64* %ret
  %load_ret = load i64, i64* %ret
  ret i64 %load_ret
}
