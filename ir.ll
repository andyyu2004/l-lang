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
  %retvar = alloca i64
  %tmp = alloca i64*
  %tmp1 = alloca i64*
  %boxed = alloca i64*
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ i64, i32 }* getelementptr ({ i64, i32 }, { i64, i32 }* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to { i64, i32 }*
  %cast_malloc_ptr = bitcast { i64, i32 }* %box to i8*
  %print_malloc_addr = call {} @print_addr(i8* %cast_malloc_ptr)
  %rc_gep = getelementptr inbounds { i64, i32 }, { i64, i32 }* %box, i32 0, i32 1
  store i32 0, i32* %rc_gep
  %box_gep = getelementptr inbounds { i64, i32 }, { i64, i32 }* %box, i32 0, i32 0
  store i64* %box_gep, i64** %tmp1
  %load_deref = load i64*, i64** %tmp1
  store i64 9, i64* %load_deref
  %load = load i64*, i64** %tmp1
  store i64* %load, i64** %tmp
  call void @"rc_retain<int>"(i64** %boxed)
  %load2 = load i64*, i64** %tmp
  store i64* %load2, i64** %boxed
  store i64 8, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

define {} @"take_box<>"(i64* %0) {
basic_blockbb0:
  %retvar = alloca {}
  %i = alloca i64*
  store i64* %0, i64** %i
  %tmp = alloca {}
  %tmp1 = alloca i64
  call void @"rc_retain<int>"(i64** %i)
  %load = load i64*, i64** %i
  %fcall = call i64 @"rc<int>"(i64* %load)
  store i64 %fcall, i64* %tmp1
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load2 = load i64, i64* %tmp1
  %fcall3 = call {} @print(i64 %load2)
  store {} %fcall3, {}* %tmp
  br label %basic_blockbb2

basic_blockbb2:                                   ; preds = %basic_blockbb1
  store {} undef, {}* %retvar
  %load_ret = load {}, {}* %retvar
  ret {} %load_ret
}

define i64 @"rc<int>"(i64* %0) {
rc_entry:
  %cast_box_ptr = bitcast i64* %0 to { i64, i64 }*
  %rc_gep = getelementptr inbounds { i64, i64 }, { i64, i64 }* %cast_box_ptr, i32 0, i32 1
  %load_refcount = load i64, i64* %rc_gep
  ret i64 %load_refcount
}

declare noalias i8* @malloc(i32)

define void @"rc_retain<int>"(i64** %0) {
rc_retain_start:
  %load_box = load i64*, i64** %0
  %cast_malloc_ptr = bitcast i64** %0 to i8*
  %print_malloc_addr = call {} @print_addr(i8* %cast_malloc_ptr)
  %cast_malloc_ptr1 = bitcast i64* %load_box to i8*
  %print_malloc_addr2 = call {} @print_addr(i8* %cast_malloc_ptr1)
  %rc_retain_box_cast = bitcast i64* %load_box to { i64, i32 }*
  %rc = getelementptr inbounds { i64, i32 }, { i64, i32 }* %rc_retain_box_cast, i32 0, i32 1
  %load_rc = load i32, i32* %rc
  %increment_rc = add i32 %load_rc, 1
  %i64rc = sext i32 %increment_rc to i64
  %print_rc = call {} @print(i64 %i64rc)
  ret void
}
