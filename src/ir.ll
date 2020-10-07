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

declare void @iprintln(i64)

declare i32 @printf()

declare void @abort()

declare void @exit(i32)

define i64 @main() {
basic_blockbb0:
  %retvar = alloca i64
  %tmp = alloca i64
  %x = alloca i64
  store i64 5, i64* %tmp
  %load = load i64, i64* %tmp
  store i64 %load, i64* %x
  %load1 = load i64, i64* %x
  store i64 %load1, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
