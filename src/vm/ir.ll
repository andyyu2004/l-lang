; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca {}
  %"mut x" = alloca i64
  %tmp1 = alloca i64
  %tmp2 = alloca i64
  %fcall = call {} @g()
  store {} %fcall, {}* %tmp
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  store i64 3, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_block2:                                     ; No predecessors!
  store i64 69, i64* %"mut x"
  %load = load i64, i64* %"mut x"
  %tmpidd = add i64 %load, 1
  store i64 %tmpidd, i64* %"mut x"
  %load3 = load i64, i64* %"mut x"
  %tmpidd4 = add i64 %load3, 1
  store i64 %tmpidd4, i64* %tmp1
  %load5 = load i64, i64* %"mut x"
  %tmpidd6 = add i64 %load5, 5
  store i64 %tmpidd6, i64* %tmp2
  switch i1 false, label %basic_block5 [
    i1 true, label %basic_block4
  ]

basic_block3:                                     ; preds = %basic_block5, %basic_block4
  %load_ret7 = load i64, i64* %retvar
  ret i64 %load_ret7

basic_block4:                                     ; preds = %basic_block2
  store i64 5, i64* %retvar
  br label %basic_block3

basic_block5:                                     ; preds = %basic_block2
  store i64 4, i64* %retvar
  br label %basic_block3
}

define {} @g() {
basic_block0:
  %retvar = alloca {}
  store {} undef, {}* %retvar
  %load_ret = load {}, {}* %retvar
  ret {} %load_ret
}
