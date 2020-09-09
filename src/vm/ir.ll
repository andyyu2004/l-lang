; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %fcall = call i64 @fib(i64 10)
  store i64 %fcall, i64* %retvar
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_block2:                                     ; No predecessors!
  %load_ret1 = load i64, i64* %retvar
  ret i64 %load_ret1
}

define i64 @fib(i64 %0) {
basic_block0:
  %retvar = alloca i64
  %n = alloca i64
  store i64 %0, i64* %n
  %tmp = alloca i64
  %tmp1 = alloca i64
  %tmp2 = alloca i64
  %tmp3 = alloca i64
  %load = load i64, i64* %n
  %icmp_lt = icmp slt i64 %load, 2
  switch i1 %icmp_lt, label %basic_block4 [
    i1 true, label %basic_block2
  ]

basic_block1:                                     ; preds = %basic_block7, %basic_block3
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_block2:                                     ; preds = %basic_block0
  %load4 = load i64, i64* %n
  store i64 %load4, i64* %retvar
  %load_ret5 = load i64, i64* %retvar
  ret i64 %load_ret5

basic_block3:                                     ; No predecessors!
  br label %basic_block1

basic_block4:                                     ; preds = %basic_block0
  %load6 = load i64, i64* %n
  %tmpisub = sub i64 %load6, 1
  store i64 %tmpisub, i64* %tmp1
  %load7 = load i64, i64* %tmp1
  %fcall = call i64 @fib(i64 %load7)
  store i64 %fcall, i64* %tmp
  br label %basic_block5

basic_block5:                                     ; preds = %basic_block4
  %load8 = load i64, i64* %n
  %tmpisub9 = sub i64 %load8, 2
  store i64 %tmpisub9, i64* %tmp3
  %load10 = load i64, i64* %tmp3
  %fcall11 = call i64 @fib(i64 %load10)
  store i64 %fcall11, i64* %tmp2
  br label %basic_block6

basic_block6:                                     ; preds = %basic_block5
  %load12 = load i64, i64* %tmp
  %load13 = load i64, i64* %tmp2
  %tmpidd = add i64 %load12, %load13
  store i64 %tmpidd, i64* %retvar
  %load_ret14 = load i64, i64* %retvar
  ret i64 %load_ret14

basic_block7:                                     ; No predecessors!
  br label %basic_block1

basic_block8:                                     ; No predecessors!
  %load_ret15 = load i64, i64* %retvar
  ret i64 %load_ret15
}
