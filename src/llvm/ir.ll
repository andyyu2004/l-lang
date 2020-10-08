; ModuleID = 'main'
source_filename = "main"

%opaque = type { i64 }

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

define i64 @main() {
basic_blockbb0:
  %retvar = alloca i64
  %tmp = alloca %opaque
  %s = alloca %opaque
  %struct_gep = getelementptr inbounds %opaque, %opaque* %tmp, i32 0, i32 0
  store i64 4, i64* %struct_gep
  %load = load %opaque, %opaque* %tmp
  store %opaque %load, %opaque* %s
  %struct_gep1 = getelementptr inbounds %opaque, %opaque* %s, i32 0, i32 0
  store i64 9, i64* %struct_gep1
  %struct_gep2 = getelementptr inbounds %opaque, %opaque* %s, i32 0, i32 0
  %load3 = load i64, i64* %struct_gep2
  store i64 %load3, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
