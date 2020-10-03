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

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca <{ i64 }>
  %s = alloca <{ i64 }>
  %struct_gep = getelementptr inbounds <{ i64 }>, <{ i64 }>* %tmp, i32 0, i32 0
  store i64 4, i64* %struct_gep
  %load = load <{ i64 }>, <{ i64 }>* %tmp
  store <{ i64 }> %load, <{ i64 }>* %s
  %struct_gep1 = getelementptr inbounds <{ i64 }>, <{ i64 }>* %s, i32 0, i32 0
  store i64 9, i64* %struct_gep1
  %struct_gep2 = getelementptr inbounds <{ i64 }>, <{ i64 }>* %s, i32 0, i32 0
  %load3 = load i64, i64* %struct_gep2
  store i64 %load3, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
