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

define %opaque @"13<int>"(i64 %0) {
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

define i64 @main() {
basic_blockbb0:
  %retvar = alloca i64
  %tmp = alloca %opaque
  %opt = alloca %opaque
  %tmp1 = alloca i1
  %tmp2 = alloca i64
  %tmp3 = alloca i1
  %k = alloca i64
  %fcall = call %opaque @"13<int>"(i64 8)
  store %opaque %fcall, %opaque* %tmp
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load = load %opaque, %opaque* %tmp
  store %opaque %load, %opaque* %opt
  br label %basic_blockbb2

basic_blockbb2:                                   ; preds = %basic_blockbb1
  store i1 true, i1* %tmp1
  %discr_gep = getelementptr inbounds %opaque, %opaque* %opt, i32 0, i32 0
  %load_discr = load i64, i64* %discr_gep
  store i64 %load_discr, i64* %tmp2
  %load4 = load i64, i64* %tmp2
  %icmp_eq = icmp eq i64 0, %load4
  store i1 %icmp_eq, i1* %tmp3
  %load5 = load i1, i1* %tmp3
  %load6 = load i1, i1* %tmp1
  %and = and i1 %load5, %load6
  store i1 %and, i1* %tmp1
  %struct_gep = getelementptr inbounds %opaque, %opaque* %opt, i32 0, i32 1
  %struct_gep7 = getelementptr inbounds { i64 }, { i64 }* %struct_gep, i32 0, i32 0
  %load8 = load i64, i64* %struct_gep7
  store i64 %load8, i64* %k
  %load9 = load i1, i1* %tmp1
  br i1 %load9, label %basic_blockbb3, label %basic_blockbb5

basic_blockbb3:                                   ; preds = %basic_blockbb2
  %load10 = load i64, i64* %k
  store i64 %load10, i64* %retvar
  br label %basic_blockbb4

basic_blockbb4:                                   ; preds = %basic_blockbb3
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_blockbb5:                                   ; preds = %basic_blockbb2
  call void @exit(i32 1)
  unreachable
}
