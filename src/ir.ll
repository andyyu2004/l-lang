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

declare i32 @printf()

declare void @abort()

declare void @exit(i32)

define i64 @main() {
basic_blockbb0:
  %retvar = alloca i64
  %fcall = call i64 @fib(i64 40)
  store i64 %fcall, i64* %retvar
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

define i64 @fib(i64 %0) {
basic_blockbb0:
  %retvar = alloca i64
  %n = alloca i64
  store i64 %0, i64* %n
  %n1 = alloca i64
  %tmp = alloca i1
  %tmp2 = alloca i1
  %tmp3 = alloca i1
  %tmp4 = alloca i1
  %tmp5 = alloca i1
  %tmp6 = alloca i64
  %tmp7 = alloca i64
  %tmp8 = alloca i64
  %tmp9 = alloca i64
  %load = load i64, i64* %n
  store i64 %load, i64* %n1
  %load10 = load i64, i64* %n1
  %icmp_lt = icmp slt i64 %load10, 2
  store i1 %icmp_lt, i1* %tmp
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  store i1 true, i1* %tmp2
  store i1 true, i1* %tmp3
  %load11 = load i1, i1* %tmp3
  %load12 = load i1, i1* %tmp
  %icmp_eq = icmp eq i1 %load11, %load12
  store i1 %icmp_eq, i1* %tmp4
  %load13 = load i1, i1* %tmp4
  %load14 = load i1, i1* %tmp2
  %and = and i1 %load13, %load14
  store i1 %and, i1* %tmp2
  %load15 = load i1, i1* %tmp2
  br i1 %load15, label %basic_blockbb2, label %basic_blockbb3

basic_blockbb2:                                   ; preds = %basic_blockbb1
  %load16 = load i64, i64* %n1
  store i64 %load16, i64* %retvar
  br label %basic_blockbb5

basic_blockbb3:                                   ; preds = %basic_blockbb1
  store i1 true, i1* %tmp5
  %load17 = load i1, i1* %tmp5
  br i1 %load17, label %basic_blockbb4, label %basic_blockbb6

basic_blockbb4:                                   ; preds = %basic_blockbb3
  %load18 = load i64, i64* %n1
  %isub = sub i64 %load18, 1
  store i64 %isub, i64* %tmp7
  %load19 = load i64, i64* %tmp7
  %fcall = call i64 @fib(i64 %load19)
  store i64 %fcall, i64* %tmp6
  br label %basic_blockbb7

basic_blockbb5:                                   ; preds = %basic_blockbb8, %basic_blockbb2
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_blockbb6:                                   ; preds = %basic_blockbb3
  call void @exit(i32 1)
  unreachable

basic_blockbb7:                                   ; preds = %basic_blockbb4
  %load20 = load i64, i64* %n1
  %isub21 = sub i64 %load20, 2
  store i64 %isub21, i64* %tmp9
  %load22 = load i64, i64* %tmp9
  %fcall23 = call i64 @fib(i64 %load22)
  store i64 %fcall23, i64* %tmp8
  br label %basic_blockbb8

basic_blockbb8:                                   ; preds = %basic_blockbb7
  %load24 = load i64, i64* %tmp6
  %load25 = load i64, i64* %tmp8
  %iadd = add i64 %load24, %load25
  store i64 %iadd, i64* %retvar
  br label %basic_blockbb5
}
