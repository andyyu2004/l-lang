; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %"mut x" = alloca i64
  %"mut y" = alloca i64
  %tmp = alloca i64
  store i64 0, i64* %"mut x"
  store i64 0, i64* %"mut y"
  store i64 6, i64* %"mut y"
  store i64 6, i64* %"mut x"
  store i64 6, i64* %tmp
  %load = load i64, i64* %"mut x"
  store i64 %load, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
