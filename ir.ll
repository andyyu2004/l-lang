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

define i64 @main() {
basic_blockbb0:
  %retvar = alloca i64
  %tmp = alloca i64*
  %tmp1 = alloca i64*
  %boxed = alloca i64*
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ i64, i32 }* getelementptr ({ i64, i32 }, { i64, i32 }* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to { i64, i32 }*
  %rc_gep = getelementptr inbounds { i64, i32 }, { i64, i32 }* %box, i32 0, i32 1
  store i32 0, i32* %rc_gep
  %box_gep = getelementptr inbounds { i64, i32 }, { i64, i32 }* %box, i32 0, i32 0
  store i64* %box_gep, i64** %tmp1
  %load_deref = load i64*, i64** %tmp1
  store i64 5, i64* %load_deref
  %load = load i64*, i64** %tmp1
  store i64* %load, i64** %tmp
  %load2 = load i64*, i64** %tmp
  store i64* %load2, i64** %boxed
  %load3 = load i64*, i64** %boxed
  %fcall = call i64 @"rc<int>"(i64* %load3)
  store i64 %fcall, i64* %retvar
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_blockbb2:                                   ; No predecessors!
  store i64 8, i64* %retvar
  %load_ret4 = load i64, i64* %retvar
  ret i64 %load_ret4
}

define i64 @"rc<int>"(i64* %0) {
rc_entry:
  %cast_box_ptr = bitcast i64* %0 to { i64, i64 }*
  %rc_gep = getelementptr inbounds { i64, i64 }, { i64, i64 }* %cast_box_ptr, i32 0, i32 1
  %load_refcount = load i64, i64* %rc_gep
  ret i64 %load_refcount
}

declare noalias i8* @malloc(i32)
