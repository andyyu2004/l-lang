; ModuleID = 'main'
source_filename = "main"

%"S<bool,int>" = type { i1, i64 }

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
  %tmp = alloca %"S<bool,int>"
  %s = alloca %"S<bool,int>"
  %fcall = call %"S<bool,int>" @"new<bool,int>"(i64 5, i1 false)
  store %"S<bool,int>" %fcall, %"S<bool,int>"* %tmp
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load = load %"S<bool,int>", %"S<bool,int>"* %tmp
  store %"S<bool,int>" %load, %"S<bool,int>"* %s
  %struct_gep = getelementptr inbounds %"S<bool,int>", %"S<bool,int>"* %s, i32 0, i32 1
  %load1 = load i64, i64* %struct_gep
  store i64 %load1, i64* %ret
  %load_ret = load i64, i64* %ret
  ret i64 %load_ret
}

define %"S<bool,int>" @"new<bool,int>"(i64 %0, i1 %1) {
basic_blockbb0:
  %ret = alloca %"S<bool,int>"
  %u = alloca i64
  store i64 %0, i64* %u
  %t = alloca i1
  store i1 %1, i1* %t
  %load = load i1, i1* %t
  %struct_gep = getelementptr inbounds %"S<bool,int>", %"S<bool,int>"* %ret, i32 0, i32 0
  store i1 %load, i1* %struct_gep
  %load1 = load i64, i64* %u
  %struct_gep2 = getelementptr inbounds %"S<bool,int>", %"S<bool,int>"* %ret, i32 0, i32 1
  store i64 %load1, i64* %struct_gep2
  %load_ret = load %"S<bool,int>", %"S<bool,int>"* %ret
  ret %"S<bool,int>" %load_ret
}
