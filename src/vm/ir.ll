; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %x = alloca i64
  %x1 = alloca i64
  store i64 5, i64* %x
  %load = load i64, i64* %x
  store i64 %load, i64* %x1
  %load2 = load i64, i64* %x1
  store i64 %load2, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
