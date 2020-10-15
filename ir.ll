; ModuleID = 'main'
source_filename = "main"

%opaque = type { i64, { i64 } }

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

define i64 @"snd<bool,int>"(i1 %0, i64 %1) {
basic_blockbb0:
  %retvar = alloca i64
  %t = alloca i1
  store i1 %0, i1* %t
  %u = alloca i64
  store i64 %1, i64* %u
  %load = load i64, i64* %u
  %load1 = load i1, i1* %t
  %fcall = call i64 @"fst<int,bool>"(i64 %load, i1 %load1)
  store i64 %fcall, i64* %retvar
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

define i64 @"main<>"() {
basic_blockbb0:
  %retvar = alloca i64
  %tmp = alloca %opaque
  %tmp1 = alloca i1
  %tmp2 = alloca i64
  %fcall = call %opaque @"Option::Some<int>"(i64 8)
  store %opaque %fcall, %opaque* %tmp
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %fcall3 = call i1 @"snd<int,bool>"(i64 9, i1 false)
  store i1 %fcall3, i1* %tmp1
  br label %basic_blockbb2

basic_blockbb2:                                   ; preds = %basic_blockbb1
  %fcall4 = call i64 @"snd<bool,int>"(i1 false, i64 9)
  store i64 %fcall4, i64* %tmp2
  br label %basic_blockbb3

basic_blockbb3:                                   ; preds = %basic_blockbb2
  store i64 8, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

define i64 @"fst<int,bool>"(i64 %0, i1 %1) {
basic_blockbb0:
  %retvar = alloca i64
  %t = alloca i64
  store i64 %0, i64* %t
  %u = alloca i1
  store i1 %1, i1* %u
  %load = load i64, i64* %t
  store i64 %load, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

define %opaque @"Option::Some<int>"(i64 %0) {
basic_blockbb0:
  %retvar = alloca %opaque
  %1 = alloca i64
  store i64 %0, i64* %1
  %discr_gep = getelementptr inbounds %opaque, %opaque* %retvar, i32 0, i32 0
  store i64 0, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque, %opaque* %retvar, i32 0, i32 1
  %load = load i64, i64* %1
  %enum_content_gep = getelementptr inbounds { i64 }, { i64 }* %enum_gep, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load_ret = load %opaque, %opaque* %retvar
  ret %opaque %load_ret
}

define i1 @"snd<int,bool>"(i64 %0, i1 %1) {
basic_blockbb0:
  %retvar = alloca i1
  %t = alloca i64
  store i64 %0, i64* %t
  %u = alloca i1
  store i1 %1, i1* %u
  %load = load i1, i1* %u
  %load1 = load i64, i64* %t
  %fcall = call i1 @"fst<bool,int>"(i1 %load, i64 %load1)
  store i1 %fcall, i1* %retvar
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load_ret = load i1, i1* %retvar
  ret i1 %load_ret
}

define i1 @"fst<bool,int>"(i1 %0, i64 %1) {
basic_blockbb0:
  %retvar = alloca i1
  %t = alloca i1
  store i1 %0, i1* %t
  %u = alloca i64
  store i64 %1, i64* %u
  %load = load i1, i1* %t
  store i1 %load, i1* %retvar
  %load_ret = load i1, i1* %retvar
  ret i1 %load_ret
}
