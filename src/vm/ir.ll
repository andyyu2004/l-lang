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
  %tmp = alloca i64***
  %ptr = alloca i64***
  %fcall = call i64*** @f1()
  store i64*** %fcall, i64**** %tmp
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  %load = load i64***, i64**** %tmp
  store i64*** %load, i64**** %ptr
  %load_box = load i64***, i64**** %ptr
  %rc_cast = bitcast i64*** %load_box to <{ i64**, i32 }>*
  %rc = getelementptr inbounds <{ i64**, i32 }>, <{ i64**, i32 }>* %rc_cast, i32 0, i32 1
  %0 = atomicrmw add i32* %rc, i32 1 seq_cst
  %load_deref = load i64***, i64**** %ptr
  %load_deref1 = load i64**, i64*** %load_deref
  %load_deref2 = load i64*, i64** %load_deref1
  %load3 = load i64, i64* %load_deref2
  store i64 %load3, i64* %retvar
  %rc_release_cast = bitcast i64**** %ptr to i8*
  %load_box4 = load i64***, i64**** %ptr
  %rc_cast5 = bitcast i64*** %load_box4 to <{ i64**, i32 }>*
  %rc6 = getelementptr inbounds <{ i64**, i32 }>, <{ i64**, i32 }>* %rc_cast5, i32 0, i32 1
  call void @rc_release(i8* %rc_release_cast, i32* %rc6)
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

define i64*** @f1() {
basic_block0:
  %retvar = alloca i64***
  %tmp = alloca i64***
  %malloccall = tail call i8* @malloc(i32 ptrtoint (<{ i64**, i32 }>* getelementptr (<{ i64**, i32 }>, <{ i64**, i32 }>* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to <{ i64**, i32 }>*
  %rc = getelementptr inbounds <{ i64**, i32 }>, <{ i64**, i32 }>* %box, i32 0, i32 1
  %0 = atomicrmw xchg i32* %rc, i32 1 seq_cst
  %rc_gep = getelementptr inbounds <{ i64**, i32 }>, <{ i64**, i32 }>* %box, i32 0, i32 0
  store i64*** %rc_gep, i64**** %tmp
  %load_box = load i64***, i64**** %tmp
  %rc_cast = bitcast i64*** %load_box to <{ i64**, i32 }>*
  %rc1 = getelementptr inbounds <{ i64**, i32 }>, <{ i64**, i32 }>* %rc_cast, i32 0, i32 1
  %1 = atomicrmw add i32* %rc1, i32 1 seq_cst
  %fcall = call i64** @f2()
  %load_deref = load i64***, i64**** %tmp
  store i64** %fcall, i64*** %load_deref
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  %load = load i64***, i64**** %tmp
  store i64*** %load, i64**** %retvar
  %load_box2 = load i64***, i64**** %retvar
  %rc_cast3 = bitcast i64*** %load_box2 to <{ i64**, i32 }>*
  %rc4 = getelementptr inbounds <{ i64**, i32 }>, <{ i64**, i32 }>* %rc_cast3, i32 0, i32 1
  %2 = atomicrmw add i32* %rc4, i32 1 seq_cst
  %rc_release_cast = bitcast i64**** %tmp to i8*
  %load_box5 = load i64***, i64**** %tmp
  %rc_cast6 = bitcast i64*** %load_box5 to <{ i64**, i32 }>*
  %rc7 = getelementptr inbounds <{ i64**, i32 }>, <{ i64**, i32 }>* %rc_cast6, i32 0, i32 1
  call void @rc_release(i8* %rc_release_cast, i32* %rc7)
  %rc_release_cast8 = bitcast i64**** %retvar to i8*
  %load_box9 = load i64***, i64**** %retvar
  %rc_cast10 = bitcast i64*** %load_box9 to <{ i64**, i32 }>*
  %rc11 = getelementptr inbounds <{ i64**, i32 }>, <{ i64**, i32 }>* %rc_cast10, i32 0, i32 1
  call void @rc_release(i8* %rc_release_cast8, i32* %rc11)
  %load_ret = load i64***, i64**** %retvar
  ret i64*** %load_ret
}

define i64** @f2() {
basic_block0:
  %retvar = alloca i64**
  %tmp = alloca i64**
  %malloccall = tail call i8* @malloc(i32 ptrtoint (<{ i64*, i32 }>* getelementptr (<{ i64*, i32 }>, <{ i64*, i32 }>* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to <{ i64*, i32 }>*
  %rc = getelementptr inbounds <{ i64*, i32 }>, <{ i64*, i32 }>* %box, i32 0, i32 1
  %0 = atomicrmw xchg i32* %rc, i32 1 seq_cst
  %rc_gep = getelementptr inbounds <{ i64*, i32 }>, <{ i64*, i32 }>* %box, i32 0, i32 0
  store i64** %rc_gep, i64*** %tmp
  %load_box = load i64**, i64*** %tmp
  %rc_cast = bitcast i64** %load_box to <{ i64*, i32 }>*
  %rc1 = getelementptr inbounds <{ i64*, i32 }>, <{ i64*, i32 }>* %rc_cast, i32 0, i32 1
  %1 = atomicrmw add i32* %rc1, i32 1 seq_cst
  %fcall = call i64* @f3()
  %load_deref = load i64**, i64*** %tmp
  store i64* %fcall, i64** %load_deref
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  %load = load i64**, i64*** %tmp
  store i64** %load, i64*** %retvar
  %load_box2 = load i64**, i64*** %retvar
  %rc_cast3 = bitcast i64** %load_box2 to <{ i64*, i32 }>*
  %rc4 = getelementptr inbounds <{ i64*, i32 }>, <{ i64*, i32 }>* %rc_cast3, i32 0, i32 1
  %2 = atomicrmw add i32* %rc4, i32 1 seq_cst
  %rc_release_cast = bitcast i64*** %tmp to i8*
  %load_box5 = load i64**, i64*** %tmp
  %rc_cast6 = bitcast i64** %load_box5 to <{ i64*, i32 }>*
  %rc7 = getelementptr inbounds <{ i64*, i32 }>, <{ i64*, i32 }>* %rc_cast6, i32 0, i32 1
  call void @rc_release(i8* %rc_release_cast, i32* %rc7)
  %rc_release_cast8 = bitcast i64*** %retvar to i8*
  %load_box9 = load i64**, i64*** %retvar
  %rc_cast10 = bitcast i64** %load_box9 to <{ i64*, i32 }>*
  %rc11 = getelementptr inbounds <{ i64*, i32 }>, <{ i64*, i32 }>* %rc_cast10, i32 0, i32 1
  call void @rc_release(i8* %rc_release_cast8, i32* %rc11)
  %load_ret = load i64**, i64*** %retvar
  ret i64** %load_ret
}

define i64* @f3() {
basic_block0:
  %retvar = alloca i64*
  %tmp = alloca i64*
  %malloccall = tail call i8* @malloc(i32 ptrtoint (<{ i64, i32 }>* getelementptr (<{ i64, i32 }>, <{ i64, i32 }>* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to <{ i64, i32 }>*
  %rc = getelementptr inbounds <{ i64, i32 }>, <{ i64, i32 }>* %box, i32 0, i32 1
  %0 = atomicrmw xchg i32* %rc, i32 1 seq_cst
  %rc_gep = getelementptr inbounds <{ i64, i32 }>, <{ i64, i32 }>* %box, i32 0, i32 0
  store i64* %rc_gep, i64** %tmp
  %load_box = load i64*, i64** %tmp
  %rc_cast = bitcast i64* %load_box to <{ i64, i32 }>*
  %rc1 = getelementptr inbounds <{ i64, i32 }>, <{ i64, i32 }>* %rc_cast, i32 0, i32 1
  %1 = atomicrmw add i32* %rc1, i32 1 seq_cst
  %load_deref = load i64*, i64** %tmp
  store i64 20, i64* %load_deref
  %load_box2 = load i64*, i64** %tmp
  %rc_cast3 = bitcast i64* %load_box2 to <{ i64, i32 }>*
  %rc4 = getelementptr inbounds <{ i64, i32 }>, <{ i64, i32 }>* %rc_cast3, i32 0, i32 1
  %2 = atomicrmw add i32* %rc4, i32 1 seq_cst
  %load = load i64*, i64** %tmp
  store i64* %load, i64** %retvar
  %load_box5 = load i64*, i64** %retvar
  %rc_cast6 = bitcast i64* %load_box5 to <{ i64, i32 }>*
  %rc7 = getelementptr inbounds <{ i64, i32 }>, <{ i64, i32 }>* %rc_cast6, i32 0, i32 1
  %3 = atomicrmw add i32* %rc7, i32 1 seq_cst
  %rc_release_cast = bitcast i64** %tmp to i8*
  %load_box8 = load i64*, i64** %tmp
  %rc_cast9 = bitcast i64* %load_box8 to <{ i64, i32 }>*
  %rc10 = getelementptr inbounds <{ i64, i32 }>, <{ i64, i32 }>* %rc_cast9, i32 0, i32 1
  call void @rc_release(i8* %rc_release_cast, i32* %rc10)
  %rc_release_cast11 = bitcast i64** %tmp to i8*
  %load_box12 = load i64*, i64** %tmp
  %rc_cast13 = bitcast i64* %load_box12 to <{ i64, i32 }>*
  %rc14 = getelementptr inbounds <{ i64, i32 }>, <{ i64, i32 }>* %rc_cast13, i32 0, i32 1
  call void @rc_release(i8* %rc_release_cast11, i32* %rc14)
  %rc_release_cast15 = bitcast i64** %retvar to i8*
  %load_box16 = load i64*, i64** %retvar
  %rc_cast17 = bitcast i64* %load_box16 to <{ i64, i32 }>*
  %rc18 = getelementptr inbounds <{ i64, i32 }>, <{ i64, i32 }>* %rc_cast17, i32 0, i32 1
  call void @rc_release(i8* %rc_release_cast15, i32* %rc18)
  %load_ret = load i64*, i64** %retvar
  ret i64* %load_ret
}

declare noalias i8* @malloc(i32)
