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

define i64 @rc(i64* %0) {
rc_entry:
  %sdf = bitcast i64* %0 to { i64, i32 }*
  %rc_gep = getelementptr inbounds { i64, i32 }, { i64, i32 }* %sdf, i32 0, i32 1
  %load_refcount = load i32, i32* %rc_gep
  %"rc->i64" = sext i32 %load_refcount to i64
  ret i64 %"rc->i64"
}

define i1 @"0<bool,int>"(i1 %0, i64 %1) {
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

define i1 @"3<int,bool>"(i64 %0, i1 %1) {
basic_blockbb0:
  %retvar = alloca i1
  %t = alloca i64
  store i64 %0, i64* %t
  %u = alloca i1
  store i1 %1, i1* %u
  %load = load i1, i1* %u
  %load1 = load i64, i64* %t
  %fcall = call i1 @"0<bool,int>"(i1 %load, i64 %load1)
  store i1 %fcall, i1* %retvar
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load_ret = load i1, i1* %retvar
  ret i1 %load_ret
}

define i64 @main() {
basic_blockbb0:
  %retvar = alloca i64
  %tmp = alloca i64
  %x = alloca i64
  %tmp1 = alloca i1
  %tmp2 = alloca i1
  %tmp3 = alloca i1
  %tmp4 = alloca i1
  %tmp5 = alloca i1
  store i64 5, i64* %tmp
  %load = load i64, i64* %tmp
  store i64 %load, i64* %x
  %load6 = load i64, i64* %x
  %fcall = call i1 @"3<int,bool>"(i64 %load6, i1 true)
  store i1 %fcall, i1* %tmp1
  br label %basic_blockbb5

basic_blockbb1:                                   ; preds = %basic_blockbb5
  store i1 true, i1* %tmp2
  store i1 true, i1* %tmp3
  %load7 = load i1, i1* %tmp3
  %load8 = load i1, i1* %tmp1
  %icmp_eq = icmp eq i1 %load7, %load8
  store i1 %icmp_eq, i1* %tmp4
  %load9 = load i1, i1* %tmp4
  %load10 = load i1, i1* %tmp2
  %and = and i1 %load9, %load10
  store i1 %and, i1* %tmp2
  %load11 = load i1, i1* %tmp2
  br i1 %load11, label %basic_blockbb2, label %basic_blockbb3

basic_blockbb2:                                   ; preds = %basic_blockbb1
  store i64 5, i64* %retvar
  br label %basic_blockbb6

basic_blockbb3:                                   ; preds = %basic_blockbb1
  store i1 true, i1* %tmp5
  %load12 = load i1, i1* %tmp5
  br i1 %load12, label %basic_blockbb4, label %basic_blockbb7

basic_blockbb4:                                   ; preds = %basic_blockbb3
  store i64 100, i64* %retvar
  br label %basic_blockbb6

basic_blockbb5:                                   ; preds = %basic_blockbb0
  br label %basic_blockbb1

basic_blockbb6:                                   ; preds = %basic_blockbb4, %basic_blockbb2
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_blockbb7:                                   ; preds = %basic_blockbb3
  call void @exit(i32 1)
  unreachable
}
