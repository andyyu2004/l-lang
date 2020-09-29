; ModuleID = 'main'
source_filename = "main"

define void @rc_release(i64* %0) {
rc_release:
  %1 = atomicrmw sub i64* %0, i64 1 seq_cst
  %rc_cmp = icmp eq i64 %1, 1
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
  %tmp = alloca i64*
  %fcall = call i64* @double_ptr()
  store i64* %fcall, i64** %tmp
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  %load_deref = load i64*, i64** %tmp
  %load = load i64, i64* %load_deref
  store i64 %load, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

define i64* @double_ptr() {
basic_block0:
  %retvar = alloca i64*
  %tmp = alloca i64*
  %malloccall = tail call i8* @malloc(i32 ptrtoint (<{ i64, i64 }>* getelementptr (<{ i64, i64 }>, <{ i64, i64 }>* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to <{ i64, i64 }>*
  %rc = getelementptr inbounds <{ i64, i64 }>, <{ i64, i64 }>* %box, i32 0, i32 0
  store i64 1, i64* %rc
  %rc_gep = getelementptr inbounds <{ i64, i64 }>, <{ i64, i64 }>* %box, i32 0, i32 1
  store i64* %rc_gep, i64** %tmp
  %load_box = load i64*, i64** %tmp
  %rc_cast = bitcast i64* %load_box to <{ i64, i64* }>*
  %rc_header_gep = getelementptr <{ i64, i64* }>, <{ i64, i64* }>* %rc_cast, i64 -1
  %rc1 = getelementptr inbounds <{ i64, i64* }>, <{ i64, i64* }>* %rc_header_gep, i32 0, i32 0
  %0 = atomicrmw add i64* %rc1, i64 1 seq_cst
  %load_deref = load i64*, i64** %tmp
  store i64 5, i64* %load_deref
  %load_box2 = load i64*, i64** %tmp
  %rc_cast3 = bitcast i64* %load_box2 to <{ i64, i64* }>*
  %rc_header_gep4 = getelementptr <{ i64, i64* }>, <{ i64, i64* }>* %rc_cast3, i64 -1
  %rc5 = getelementptr inbounds <{ i64, i64* }>, <{ i64, i64* }>* %rc_header_gep4, i32 0, i32 0
  %1 = atomicrmw add i64* %rc5, i64 1 seq_cst
  %load = load i64*, i64** %tmp
  store i64* %load, i64** %retvar
  %load_box6 = load i64*, i64** %retvar
  %rc_cast7 = bitcast i64* %load_box6 to <{ i64, i64* }>*
  %rc_header_gep8 = getelementptr <{ i64, i64* }>, <{ i64, i64* }>* %rc_cast7, i64 -1
  %rc9 = getelementptr inbounds <{ i64, i64* }>, <{ i64, i64* }>* %rc_header_gep8, i32 0, i32 0
  %2 = atomicrmw add i64* %rc9, i64 1 seq_cst
  %load_box10 = load i64*, i64** %tmp
  %rc_cast11 = bitcast i64* %load_box10 to <{ i64, i64* }>*
  %rc_header_gep12 = getelementptr <{ i64, i64* }>, <{ i64, i64* }>* %rc_cast11, i64 -1
  %rc13 = getelementptr inbounds <{ i64, i64* }>, <{ i64, i64* }>* %rc_header_gep12, i32 0, i32 0
  call void @rc_release(i64* %rc13)
  %load_box14 = load i64*, i64** %tmp
  %rc_cast15 = bitcast i64* %load_box14 to <{ i64, i64* }>*
  %rc_header_gep16 = getelementptr <{ i64, i64* }>, <{ i64, i64* }>* %rc_cast15, i64 -1
  %rc17 = getelementptr inbounds <{ i64, i64* }>, <{ i64, i64* }>* %rc_header_gep16, i32 0, i32 0
  call void @rc_release(i64* %rc17)
  %load_box18 = load i64*, i64** %retvar
  %rc_cast19 = bitcast i64* %load_box18 to <{ i64, i64* }>*
  %rc_header_gep20 = getelementptr <{ i64, i64* }>, <{ i64, i64* }>* %rc_cast19, i64 -1
  %rc21 = getelementptr inbounds <{ i64, i64* }>, <{ i64, i64* }>* %rc_header_gep20, i32 0, i32 0
  call void @rc_release(i64* %rc21)
  %load_ret = load i64*, i64** %retvar
  ret i64* %load_ret
}

declare noalias i8* @malloc(i32)
