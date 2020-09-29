; ModuleID = 'main'
source_filename = "main"

define void @rc_release(i64* %0) {
rc_release:
  %1 = atomicrmw sub i64* %0, i64 1 seq_cst
  %rc_cmp = icmp ule i64 %1, 1
  br i1 %rc_cmp, label %free, label %ret

free:                                             ; preds = %rc_release
  %2 = bitcast i64* %0 to i8*
  tail call void @free(i8* %2)
  ret void

ret:                                              ; preds = %rc_release
  ret void
}

declare void @free(i8*)

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca <{ i64, i1 }>*
  %tmp1 = alloca <{ i64, i1 }>*
  %boxed = alloca <{ i64, i1 }>*
  %malloccall = tail call i8* @malloc(i32 ptrtoint (<{ i64, <{ i64, i1 }> }>* getelementptr (<{ i64, <{ i64, i1 }> }>, <{ i64, <{ i64, i1 }> }>* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to <{ i64, <{ i64, i1 }> }>*
  %rc = getelementptr inbounds <{ i64, <{ i64, i1 }> }>, <{ i64, <{ i64, i1 }> }>* %box, i32 0, i32 0
  store i64 1, i64* %rc
  %rc_gep = getelementptr inbounds <{ i64, <{ i64, i1 }> }>, <{ i64, <{ i64, i1 }> }>* %box, i32 0, i32 1
  store <{ i64, i1 }>* %rc_gep, <{ i64, i1 }>** %tmp1
  %load = load <{ i64, i1 }>*, <{ i64, i1 }>** %tmp1
  store <{ i64, i1 }>* %load, <{ i64, i1 }>** %tmp
  %load2 = load <{ i64, i1 }>*, <{ i64, i1 }>** %tmp
  store <{ i64, i1 }>* %load2, <{ i64, i1 }>** %boxed
  store i64 8, i64* %retvar
  %load_box = load <{ i64, i1 }>*, <{ i64, i1 }>** %tmp1
  %rc_cast = bitcast <{ i64, i1 }>* %load_box to <{ i64, <{ i64, i1 }>* }>*
  %rc_header_gep = getelementptr <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_cast, i64 -1
  %rc3 = getelementptr inbounds <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_header_gep, i32 0, i32 0
  call void @rc_release(i64* %rc3)
  %load_box4 = load <{ i64, i1 }>*, <{ i64, i1 }>** %tmp
  %rc_cast5 = bitcast <{ i64, i1 }>* %load_box4 to <{ i64, <{ i64, i1 }>* }>*
  %rc_header_gep6 = getelementptr <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_cast5, i64 -1
  %rc7 = getelementptr inbounds <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_header_gep6, i32 0, i32 0
  call void @rc_release(i64* %rc7)
  %load_box8 = load <{ i64, i1 }>*, <{ i64, i1 }>** %boxed
  %rc_cast9 = bitcast <{ i64, i1 }>* %load_box8 to <{ i64, <{ i64, i1 }>* }>*
  %rc_header_gep10 = getelementptr <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_cast9, i64 -1
  %rc11 = getelementptr inbounds <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_header_gep10, i32 0, i32 0
  call void @rc_release(i64* %rc11)
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

declare noalias i8* @malloc(i32)
