; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca <{ i64 }>
  %jen = alloca <{ i64 }>
  %tmp1 = alloca <{ i64, i64 }>
  %x = alloca i64
  %y = alloca i64
  %struct_gep = getelementptr inbounds <{ i64 }>, <{ i64 }>* %tmp, i32 0, i32 0
  store i64 69, i64* %struct_gep
  %load = load <{ i64 }>, <{ i64 }>* %tmp
  store <{ i64 }> %load, <{ i64 }>* %jen
  %struct_gep2 = getelementptr inbounds <{ i64 }>, <{ i64 }>* %jen, i32 0, i32 0
  %load3 = load i64, i64* %struct_gep2
  store i64 %load3, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_block1:                                     ; No predecessors!
  %struct_gep4 = getelementptr inbounds <{ i64, i64 }>, <{ i64, i64 }>* %tmp1, i32 0, i32 0
  store i64 1, i64* %struct_gep4
  %struct_gep5 = getelementptr inbounds <{ i64, i64 }>, <{ i64, i64 }>* %tmp1, i32 0, i32 1
  store i64 5, i64* %struct_gep5
  %struct_gep6 = getelementptr inbounds <{ i64, i64 }>, <{ i64, i64 }>* %tmp1, i32 0, i32 0
  %load7 = load i64, i64* %struct_gep6
  store i64 %load7, i64* %x
  %struct_gep8 = getelementptr inbounds <{ i64, i64 }>, <{ i64, i64 }>* %tmp1, i32 0, i32 1
  %load9 = load i64, i64* %struct_gep8
  store i64 %load9, i64* %y
  %load10 = load i64, i64* %x
  store i64 %load10, i64* %retvar
  %load_ret11 = load i64, i64* %retvar
  ret i64 %load_ret11

basic_block2:                                     ; No predecessors!
  %load_ret12 = load i64, i64* %retvar
  ret i64 %load_ret12
}

define i64 @fib(i64 %0) {
basic_block0:
  %retvar = alloca i64
  %n = alloca i64
  store i64 %0, i64* %n
  %n1 = alloca i64
  %tmp = alloca i1
  %tmp2 = alloca i64
  %tmp3 = alloca i64
  %tmp4 = alloca i64
  %tmp5 = alloca i64
  %load = load i64, i64* %n
  store i64 %load, i64* %n1
  %load6 = load i64, i64* %n1
  %icmp_lt = icmp slt i64 %load6, 2
  store i1 %icmp_lt, i1* %tmp
  %load7 = load i1, i1* %tmp
  switch i1 %load7, label %basic_block3 [
    i1 true, label %basic_block2
  ]

basic_block1:                                     ; preds = %basic_block5, %basic_block2
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_block2:                                     ; preds = %basic_block0
  %load8 = load i64, i64* %n1
  store i64 %load8, i64* %retvar
  br label %basic_block1

basic_block3:                                     ; preds = %basic_block0
  %load9 = load i64, i64* %n1
  %tmpisub = sub i64 %load9, 1
  store i64 %tmpisub, i64* %tmp3
  %load10 = load i64, i64* %tmp3
  %fcall = call i64 @fib(i64 %load10)
  store i64 %fcall, i64* %tmp2
  br label %basic_block4

basic_block4:                                     ; preds = %basic_block3
  %load11 = load i64, i64* %n1
  %tmpisub12 = sub i64 %load11, 2
  store i64 %tmpisub12, i64* %tmp5
  %load13 = load i64, i64* %tmp5
  %fcall14 = call i64 @fib(i64 %load13)
  store i64 %fcall14, i64* %tmp4
  br label %basic_block5

basic_block5:                                     ; preds = %basic_block4
  %load15 = load i64, i64* %tmp2
  %load16 = load i64, i64* %tmp4
  %tmpidd = add i64 %load15, %load16
  store i64 %tmpidd, i64* %retvar
  br label %basic_block1
}
