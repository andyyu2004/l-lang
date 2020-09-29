; ModuleID = 'main'
source_filename = "main"

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
  %load_box = load <{ i64, i1 }>*, <{ i64, i1 }>** %tmp1
  %rc_cast = bitcast <{ i64, i1 }>* %load_box to <{ i64, <{ i64, i1 }>* }>*
  %rc_header_gep = getelementptr <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_cast, i64 -1
  %rc2 = getelementptr inbounds <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_header_gep, i32 0, i32 0
  %0 = atomicrmw add i64* %rc2, i64 1 seq_cst
  %load = load <{ i64, i1 }>*, <{ i64, i1 }>** %tmp1
  store <{ i64, i1 }>* %load, <{ i64, i1 }>** %tmp
  %load_box3 = load <{ i64, i1 }>*, <{ i64, i1 }>** %tmp
  %rc_cast4 = bitcast <{ i64, i1 }>* %load_box3 to <{ i64, <{ i64, i1 }>* }>*
  %rc_header_gep5 = getelementptr <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_cast4, i64 -1
  %rc6 = getelementptr inbounds <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_header_gep5, i32 0, i32 0
  %1 = atomicrmw add i64* %rc6, i64 1 seq_cst
  %load7 = load <{ i64, i1 }>*, <{ i64, i1 }>** %tmp
  store <{ i64, i1 }>* %load7, <{ i64, i1 }>** %boxed
  %load_box8 = load <{ i64, i1 }>*, <{ i64, i1 }>** %boxed
  %rc_cast9 = bitcast <{ i64, i1 }>* %load_box8 to <{ i64, <{ i64, i1 }>* }>*
  %rc_header_gep10 = getelementptr <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_cast9, i64 -1
  %rc11 = getelementptr inbounds <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_header_gep10, i32 0, i32 0
  %2 = atomicrmw add i64* %rc11, i64 1 seq_cst
  store i64 8, i64* %retvar
  %load_box12 = load <{ i64, i1 }>*, <{ i64, i1 }>** %tmp1
  %rc_cast13 = bitcast <{ i64, i1 }>* %load_box12 to <{ i64, <{ i64, i1 }>* }>*
  %rc_header_gep14 = getelementptr <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_cast13, i64 -1
  %rc15 = getelementptr inbounds <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_header_gep14, i32 0, i32 0
  %load_box16 = load <{ i64, i1 }>*, <{ i64, i1 }>** %tmp
  %rc_cast17 = bitcast <{ i64, i1 }>* %load_box16 to <{ i64, <{ i64, i1 }>* }>*
  %rc_header_gep18 = getelementptr <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_cast17, i64 -1
  %rc19 = getelementptr inbounds <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_header_gep18, i32 0, i32 0
  %load_box20 = load <{ i64, i1 }>*, <{ i64, i1 }>** %boxed
  %rc_cast21 = bitcast <{ i64, i1 }>* %load_box20 to <{ i64, <{ i64, i1 }>* }>*
  %rc_header_gep22 = getelementptr <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_cast21, i64 -1
  %rc23 = getelementptr inbounds <{ i64, <{ i64, i1 }>* }>, <{ i64, <{ i64, i1 }>* }>* %rc_header_gep22, i32 0, i32 0
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

declare noalias i8* @malloc(i32)
