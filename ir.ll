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

declare i32 @printf(i8*)

declare void @abort()

declare void @exit(i32)

define i64 @rc(i64* %0) {
rc_entry:
  %sdf = bitcast i64* %0 to { i64, i32 }*
  %rc_gep = getelementptr inbounds { i64, i32 }, { i64, i32 }* %sdf, i32 0, i32 1
  %load_refcount = load i32, i32* %rc_gep
  %"rc->i64" = sext i32 %load_refcount to i64
  ret i64 %"rc->i64"
}

define i64 @"id<int,bool>"(i64 %0, i1 %1) {
basic_blockbb0:
  %retvar = alloca i64
  %t = alloca i64
  store i64 %0, i64* %t
  %t1 = alloca i64
  store i1 %1, i64* %t1
  %u = alloca i1
  %u2 = alloca i1
  %load = load i64, i64* %t
  store i64 %load, i64* %t1
  %load3 = load i1, i1* %u
  store i1 %load3, i1* %u2
  %load4 = load i64, i64* %t1
  store i64 %load4, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

define i64 @main() {
basic_blockbb0:
  %retvar = alloca i64
  %fcall = call i64 @"id<int,bool>"(i64 4, i1 false)
  store i64 %fcall, i64* %retvar
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
