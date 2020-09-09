; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca { i64, i1 }
  %fcall = call { i64, i1 } @mktuple()
  store { i64, i1 } %fcall, { i64, i1 }* %tmp
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  store i64 0, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

define { i64, i1 } @mktuple() {
basic_block0:
  %retvar = alloca { i64, i1 }
  store { i64, i1 } { i64 30, i1 true }, { i64, i1 }* %retvar
  %load_ret = load { i64, i1 }, { i64, i1 }* %retvar
  ret { i64, i1 } %load_ret
}
